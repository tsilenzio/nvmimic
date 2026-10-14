//! Recorded outcomes of official nvm, so an everyday run only has to run nvmimic.
//!
//! Official nvm at the pinned version gives the same answer every time for the
//! same case, shell and machine. So the first run records what it did for
//! each case under each shell, one file per pair in `target/parity-golden/`,
//! and later runs read the file instead of running nvm again.
//!
//! Every file carries a key made from everything the outcome depends on: the
//! case itself, the vendored nvm, the shell program, the operating system
//! version, and the parts of this harness that shape a run. When any of them
//! changes the key no longer matches, and the pair is recorded again. A full
//! run (`cargo xtask parity --full`) records every pair again regardless, and
//! says so when a recording it replaces had gone stale.
//!
//! The recordings stay on the machine that made them. What nvm prints depends
//! on the machine, so they are not something to commit.

use crate::cases::Case;
use crate::harness::{self, Outcome, Shell};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

/// Bumped when the shape of a recording changes.
const FORMAT: &str = "1";

/// Harness sources that decide what a run of nvm looks like. A change to any
/// of them makes every recording stale.
const SHAPING_SOURCES: &[&str] = &["src/harness.rs", "src/mirror.rs", "src/suite.rs"];

pub fn dir() -> PathBuf {
    harness::target_dir().join("parity-golden")
}

fn hex(digest: impl AsRef<[u8]>) -> String {
    digest
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Hashes every file under `root`: its path, whether it can be run, and its
/// contents.
fn hash_tree(root: &Path, hasher: &mut Sha256) {
    use std::os::unix::fs::PermissionsExt;
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            match fs::symlink_metadata(&path) {
                Ok(meta) if meta.is_dir() => pending.push(path),
                Ok(meta) => files.push((path, meta.permissions().mode() & 0o111 != 0)),
                Err(_) => {}
            }
        }
    }
    files.sort();
    for (path, runnable) in files {
        hasher.update(
            path.strip_prefix(root)
                .unwrap()
                .as_os_str()
                .as_encoded_bytes(),
        );
        hasher.update([0, runnable as u8]);
        hasher.update(fs::read(&path).unwrap_or_default());
        hasher.update([0]);
    }
}

/// The part of every key that is the same for the whole run.
fn base() -> &'static str {
    static BASE: OnceLock<String> = OnceLock::new();
    BASE.get_or_init(|| {
        let mut hasher = Sha256::new();
        hasher.update(FORMAT);
        // The kernel's version string changes with every OS update, and the
        // programs nvm calls (curl, tar, awk and the rest) change with it.
        let os = Command::new("uname").arg("-srvm").output();
        hasher.update(os.map(|output| output.stdout).unwrap_or_default());
        hash_tree(&crate::reference_dir(), &mut hasher);
        // Recordings hold the hashes of archives downloaded from the mirror.
        hasher.update(&crate::mirror::shared().identity);
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        for source in SHAPING_SOURCES {
            hasher.update(fs::read(manifest.join(source)).unwrap());
        }
        hex(hasher.finalize())
    })
}

/// The hash of a shell's program file, which changes when the shell does.
fn shell_hash(shell: &Shell) -> String {
    static HASHES: Mutex<BTreeMap<PathBuf, String>> = Mutex::new(BTreeMap::new());
    HASHES
        .lock()
        .unwrap()
        .entry(shell.path.clone())
        .or_insert_with(|| hex(Sha256::digest(fs::read(&shell.path).unwrap_or_default())))
        .clone()
}

fn key(case: &Case, shell: &Shell) -> String {
    let mut hasher = Sha256::new();
    for part in [base(), &shell.name, &shell_hash(shell), &case.fingerprint] {
        hasher.update(part);
        hasher.update([0]);
    }
    hex(hasher.finalize())
}

fn file(case: &Case, shell: &Shell) -> PathBuf {
    dir().join(&case.id).join(format!("{}.json", shell.name))
}

/// Reads a recording, if the file is there and was made with this key.
fn read(path: &Path, key: &str) -> Option<Outcome> {
    let text = fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    if value["key"].as_str()? != key {
        return None;
    }
    let state = value["state"]
        .as_object()?
        .iter()
        .map(|(name, value)| Some((name.clone(), value.as_str()?.to_string())))
        .collect::<Option<BTreeMap<String, String>>>()?;
    Some(Outcome {
        stdout: value["stdout"].as_str()?.to_string(),
        stderr: value["stderr"].as_str()?.to_string(),
        code: value["code"].as_i64().map(|code| code as i32),
        timed_out: false,
        state,
        launches: Vec::new(),
        covered: BTreeSet::new(),
        elapsed: Duration::ZERO,
        phases: None,
    })
}

/// Writes a recording. A run that timed out says nothing about nvm, so it
/// is not kept.
fn write(path: &Path, key: &str, label: &str, shell: &Shell, outcome: &Outcome) {
    if outcome.timed_out {
        return;
    }
    let state: Map<String, Value> = outcome
        .state
        .iter()
        .map(|(name, value)| (name.clone(), Value::String(value.clone())))
        .collect();
    let value = json!({
        "key": key,
        "of": label,
        "shell": shell.name,
        "nvm": nvmimic::pin::TAG,
        "took_ms": outcome.elapsed.as_millis() as u64,
        "code": outcome.code,
        "stdout": outcome.stdout,
        "stderr": outcome.stderr,
        "state": state,
    });
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, serde_json::to_string_pretty(&value).unwrap() + "\n").unwrap();
}

/// What official nvm did for this pair when it was last recorded, if the
/// recording is still good for the case, shell and machine as they are now.
pub fn load(case: &Case, shell: &Shell) -> Option<Outcome> {
    read(&file(case, shell), &key(case, shell))
}

/// Records what official nvm did for this pair, and makes sure the state the
/// case's body started from is on record too.
pub fn store(case: &Case, shell: &Shell, outcome: &Outcome) {
    write(
        &file(case, shell),
        &key(case, shell),
        &case.id,
        shell,
        outcome,
    );
    before_body(case, shell);
}

/// The shell's state after a case's sourcing and setup, before its body.
/// Many cases share a setup, so this is recorded once per setup and shell.
fn before_body(case: &Case, shell: &Shell) -> Arc<BTreeMap<String, String>> {
    type Slot = Arc<OnceLock<Arc<BTreeMap<String, String>>>>;
    static KNOWN: Mutex<BTreeMap<String, Slot>> = Mutex::new(BTreeMap::new());

    let mut hasher = Sha256::new();
    let start = match &case.suite {
        Some(_) => "suite".to_string(),
        None => format!("{}\0{}\0{}", case.source, case.setup, case.plain_files),
    };
    for part in [base(), &shell.name, &shell_hash(shell), &start] {
        hasher.update(part);
        hasher.update([0]);
    }
    let key = hex(hasher.finalize());

    // The lock covers finding the slot only, so different setups are worked
    // out side by side while a second request for the same one waits.
    let slot = KNOWN
        .lock()
        .unwrap()
        .entry(key.clone())
        .or_default()
        .clone();
    slot.get_or_init(|| {
        let path = dir()
            .join("before-body")
            .join(format!("{}.json", &key[..24]));
        let outcome = read(&path, &key).unwrap_or_else(|| {
            let outcome = harness::before_body(shell, case);
            write(&path, &key, "the start of a case's body", shell, &outcome);
            outcome
        });
        Arc::new(outcome.state)
    })
    .clone()
}

/// State that belongs to the shell and not to anything nvm did.
const SHELL_OWN: &[&str] = &[
    // Only some shells can list their functions.
    "functions",
    // The text of the script the shell was started with.
    "var:BASH_EXECUTION_STRING",
    "var:ZSH_EXECUTION_STRING",
    // zsh keeps these arrays in step with PATH, MANPATH and the like.
    "var:path",
    "var:manpath",
    "var:fpath",
    "var:cdpath",
    "var:module_path",
    "var:psvar",
    // zsh's name for `$?`.
    "var:status",
];

/// A value from a shell's `set` listing without its quoting, so the same
/// value compares equal across shells that quote differently.
fn unquoted(value: &str) -> String {
    match value
        .strip_prefix('\'')
        .and_then(|rest| rest.strip_suffix('\''))
    {
        Some(inner) => inner.replace(r"'\''", "'"),
        None => value.to_string(),
    }
}

/// What a case's body did in one shell: what it printed, its exit status,
/// and every variable and file it left different from how it found them.
fn body_effect(case: &Case, shell: &Shell, outcome: &Outcome) -> BTreeMap<String, String> {
    let before = before_body(case, shell);
    let mut effect: BTreeMap<String, String> = outcome
        .state
        .iter()
        .filter(|(name, value)| before.get(*name) != Some(*value))
        .map(|(name, value)| match name.starts_with("var:") {
            true => (name.clone(), unquoted(value)),
            false => (name.clone(), value.clone()),
        })
        .collect();
    for name in before.keys() {
        if !outcome.state.contains_key(name) {
            effect.insert(name.clone(), "gone".to_string());
        }
    }
    effect.retain(|name, _| !SHELL_OWN.contains(&name.as_str()));
    effect.insert("stdout".to_string(), outcome.stdout.clone());
    effect.insert("stderr".to_string(), outcome.stderr.clone());
    effect.insert("exit".to_string(), format!("{:?}", outcome.code));
    effect
}

/// The parts of what a case's body did under official nvm that are not the same
/// in every shell. A case with none tells little more under four shells than
/// under one.
pub fn shell_differences(case: &Case, recorded: &[(&Shell, &Outcome)]) -> BTreeSet<String> {
    let effects: Vec<BTreeMap<String, String>> = recorded
        .iter()
        .map(|(shell, outcome)| body_effect(case, shell, outcome))
        .collect();
    let mut differing = BTreeSet::new();
    for other in effects.iter().skip(1) {
        let first = &effects[0];
        for name in first.keys().chain(other.keys()) {
            if first.get(name) != other.get(name) {
                differing.insert(name.clone());
            }
        }
    }
    differing
}

/// A number that goes up by one with every everyday run, so a case that runs
/// under one shell gets a different shell each time. `NVMIMIC_PARITY_ROTATION`
/// pins it, to repeat an earlier run.
pub fn rotation() -> u64 {
    static ROTATION: OnceLock<u64> = OnceLock::new();
    *ROTATION.get_or_init(|| {
        if let Some(pinned) = std::env::var("NVMIMIC_PARITY_ROTATION")
            .ok()
            .and_then(|value| value.parse().ok())
        {
            return pinned;
        }
        let path = dir().join("rotation");
        let last: u64 = fs::read_to_string(&path)
            .ok()
            .and_then(|text| text.trim().parse().ok())
            .unwrap_or(0);
        let _ = fs::create_dir_all(dir());
        let _ = fs::write(&path, format!("{}\n", last + 1));
        last + 1
    })
}

/// Which of a case's shells runs when it runs under one. Different cases
/// start at different shells, so every run still uses all of them.
pub fn pick(case: &Case, rotation: u64) -> usize {
    let spread = case.id.bytes().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ byte as u64).wrapping_mul(0x100000001b3)
    });
    (spread.wrapping_add(rotation) % case.shells.len() as u64) as usize
}
