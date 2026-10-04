//! `cargo xtask parity --changed`: run only the cases a change can affect.
//!
//! After a parity run passes, the hash of every file a run depends on is
//! saved in `target/parity-passed.json`. The next `--changed` run compares
//! the files with that list. A changed Rust file in the library names the
//! nvm functions it ports in `nvm: <name>` comments, and each case lists the
//! nvm functions its body runs in `# covers:` headers. The cases to run are
//! the ones that cover a function ported in a changed file.
//!
//! Anything this cannot place makes every case run: a changed file with no
//! `nvm:` comment, the command line crate, the shell scripts, the harness,
//! the vendored nvm. So the worst a wrong guess costs is time.
//!
//! One guess can still be wrong the other way. nvmimic's port of one nvm
//! function may call the port of another where nvm itself does not, and a
//! case covering only the first is then not picked. The everyday run without
//! `--changed` runs every case and catches that.

use nvmimic_parity::cases;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Files and directories a parity run depends on, from the repository root.
const WATCHED: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "nvm-pin.toml",
    "crates",
    "shell",
    "tests/differential",
    "tests/reference",
];

/// Files under `cases/` that describe results and change nothing about a run.
const DESCRIPTIVE: &[&str] = &["KNOWN_FAILING", "NONDETERMINISTIC", "NVM_SUITE_COVERS"];

const LIBRARY: &str = "crates/nvmimic/src/";
const CASES: &str = "tests/differential/cases/";

pub enum Affected {
    /// Every case, with the reason.
    Everything(String),
    /// These cases, by id. May be empty.
    Cases(BTreeSet<String>),
}

/// What the tree looked like: each file's hash, and for library sources the
/// nvm functions the file ports.
struct Snapshot {
    hashes: BTreeMap<String, String>,
    ports: BTreeMap<String, BTreeSet<String>>,
}

fn stamp() -> PathBuf {
    crate::repo_root().join("target/parity-passed.json")
}

fn walk(path: &Path, out: &mut Vec<PathBuf>) {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => {
            for entry in fs::read_dir(path).into_iter().flatten().flatten() {
                walk(&entry.path(), out);
            }
        }
        Ok(meta) if meta.is_file() => out.push(path.to_path_buf()),
        _ => {}
    }
}

fn current() -> Snapshot {
    let root = crate::repo_root();
    let mut files = Vec::new();
    for watched in WATCHED {
        walk(&root.join(watched), &mut files);
    }
    let mut hashes = BTreeMap::new();
    let mut ports = BTreeMap::new();
    for file in files {
        let name = file.strip_prefix(&root).unwrap().display().to_string();
        let bytes = fs::read(&file).unwrap_or_default();
        let hash: String = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        if name.starts_with(LIBRARY) && name.ends_with(".rs") {
            let units = crate::trace::anchored_units(&String::from_utf8_lossy(&bytes))
                .into_iter()
                .map(|(_, unit)| unit)
                .collect();
            ports.insert(name.clone(), units);
        }
        hashes.insert(name, hash);
    }
    Snapshot { hashes, ports }
}

fn saved() -> Option<Snapshot> {
    let value: Value = serde_json::from_str(&fs::read_to_string(stamp()).ok()?).ok()?;
    let hashes = value["hashes"]
        .as_object()?
        .iter()
        .map(|(name, hash)| Some((name.clone(), hash.as_str()?.to_string())))
        .collect::<Option<_>>()?;
    let ports = value["ports"]
        .as_object()?
        .iter()
        .map(|(name, units)| {
            let units = units
                .as_array()?
                .iter()
                .map(|unit| unit.as_str().map(str::to_string))
                .collect::<Option<_>>()?;
            Some((name.clone(), units))
        })
        .collect::<Option<_>>()?;
    Some(Snapshot { hashes, ports })
}

/// Records the tree as it is now as one that passed.
pub fn mark_passed() {
    let now = current();
    let hashes: Map<String, Value> = now
        .hashes
        .into_iter()
        .map(|(name, hash)| (name, Value::String(hash)))
        .collect();
    let ports: Map<String, Value> = now
        .ports
        .into_iter()
        .map(|(name, units)| (name, json!(units)))
        .collect();
    let text = serde_json::to_string_pretty(&json!({ "hashes": hashes, "ports": ports })).unwrap();
    let _ = fs::write(stamp(), text + "\n");
}

/// The cases a `--changed` run has to run.
pub fn affected() -> Affected {
    let Some(before) = saved() else {
        return Affected::Everything("no earlier passing run is on record".to_string());
    };
    let now = current();
    let names: BTreeSet<&String> = before.hashes.keys().chain(now.hashes.keys()).collect();
    let changed: Vec<&String> = names
        .into_iter()
        .filter(|name| before.hashes.get(*name) != now.hashes.get(*name))
        .collect();

    let all = cases::load_all();
    let mut units: BTreeSet<&String> = BTreeSet::new();
    let mut ids: BTreeSet<String> = BTreeSet::new();
    for name in &changed {
        if let Some(file) = name.strip_prefix(CASES) {
            if DESCRIPTIVE.contains(&file) {
                continue;
            }
            if !file.ends_with(".sh") {
                return Affected::Everything(format!("{name} changed"));
            }
            let path = crate::repo_root().join(name);
            ids.extend(
                all.iter()
                    .filter(|case| case.file.as_deref() == Some(path.as_path()))
                    .map(|case| case.id.clone()),
            );
        } else if name.starts_with(LIBRARY) && name.ends_with(".rs") {
            let ported: BTreeSet<&String> = [&before.ports, &now.ports]
                .iter()
                .filter_map(|ports| ports.get(*name))
                .flatten()
                .collect();
            if ported.is_empty() {
                return Affected::Everything(format!("{name} changed and names no nvm function"));
            }
            units.extend(ported);
        } else {
            return Affected::Everything(format!("{name} changed"));
        }
    }

    for case in &all {
        // A case with no measured covers could be affected by anything.
        let unknown = case.covers.is_empty() && !units.is_empty();
        if unknown || case.covers.iter().any(|unit| units.contains(unit)) {
            ids.insert(case.id.clone());
        }
    }
    eprintln!(
        "{} file(s) changed since the last passing run, {} of {} cases can be affected",
        changed.len(),
        ids.len(),
        all.len()
    );
    Affected::Cases(ids)
}
