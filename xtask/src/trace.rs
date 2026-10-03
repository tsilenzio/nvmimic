//! `cargo xtask trace`: which parts of the pinned nvm.sh nvmimic ports, where,
//! and which parity cases cover them.
//!
//! Rust code names the nvm function it follows with an `nvm: <name>` line in
//! a comment. Case files name what they exercise with `# covers: <names>`.
//! Until coverage is complete this only reports. `--strict` fails when a
//! unit is neither ported nor copied into `shell/nvm.sh`.

use crate::nvmsh::{self, Unit};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub fn run(args: &[&str]) -> ExitCode {
    let strict = args.contains(&"--strict");
    let map = match Map::build() {
        Ok(map) => map,
        Err(message) => {
            eprintln!("trace: {message}");
            return ExitCode::FAILURE;
        }
    };

    let mut ported = 0;
    let mut shell = 0;
    let mut missing: Vec<&Unit> = Vec::new();
    println!("| nvm.sh | lines | nvmimic | cases |");
    println!("|---|---|---|---|");
    for unit in &map.units {
        // A message in text/ alone does not make a port.
        let anchors = map
            .anchors
            .get(&unit.name)
            .filter(|places| places.iter().any(|place| !place.contains("/src/text/")));
        let verbatim = crate::gen_shell::is_verbatim(&unit.name);
        let place = match (anchors, verbatim) {
            (Some(places), _) => {
                ported += 1;
                places.join(", ")
            }
            (None, true) => {
                shell += 1;
                "shell/nvm.sh (copied)".to_string()
            }
            (None, false) => {
                missing.push(unit);
                "real nvm (fallback)".to_string()
            }
        };
        let cases = map
            .covers
            .get(&unit.name)
            .map_or(String::new(), |ids| ids.join(" "));
        println!(
            "| `{}` | {}-{} | {place} | {cases} |",
            unit.name, unit.start, unit.end
        );
    }
    println!();
    println!(
        "{} units: {ported} ported, {shell} copied into shell/nvm.sh, {} still in real nvm",
        map.units.len(),
        missing.len()
    );
    if strict && !missing.is_empty() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

pub struct Map {
    pub units: Vec<Unit>,
    /// nvm unit name to `file:line` of each Rust item that ports it.
    pub anchors: BTreeMap<String, Vec<String>>,
    /// nvm unit name to the ids of cases that name it in `# covers:`.
    pub covers: BTreeMap<String, Vec<String>>,
}

impl Map {
    pub fn build() -> Result<Map, String> {
        let source =
            fs::read_to_string(crate::reference_dir().join("nvm.sh")).map_err(|e| e.to_string())?;
        let root = crate::repo_root();
        Ok(Map {
            units: nvmsh::units(&source),
            anchors: anchors(&root.join("crates/nvmimic/src"), &root),
            covers: covers(&root.join("tests/differential/cases")),
        })
    }
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn anchors(dir: &Path, root: &Path) -> BTreeMap<String, Vec<String>> {
    let mut files = Vec::new();
    rust_files(dir, &mut files);
    files.sort();
    let mut anchors: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in files {
        let text = fs::read_to_string(&file).unwrap_or_default();
        let shown = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .display()
            .to_string();
        for (index, line) in text.lines().enumerate() {
            let Some(comment) = line.trim_start().strip_prefix("//") else {
                continue;
            };
            let comment = comment.trim_start_matches(['/', '!']).trim();
            let Some(names) = comment.strip_prefix("nvm: ") else {
                continue;
            };
            let is_unit = |name: &&str| {
                !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || "_:-*".contains(c))
            };
            for name in names.split([' ', ',']).filter(is_unit) {
                anchors
                    .entry(name.to_string())
                    .or_default()
                    .push(format!("{shown}:{}", index + 1));
            }
        }
    }
    anchors
}

fn covers(dir: &Path) -> BTreeMap<String, Vec<String>> {
    let mut covers: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return covers;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    for path in paths
        .into_iter()
        .filter(|path| path.extension().is_some_and(|e| e == "sh"))
    {
        let text = fs::read_to_string(&path).unwrap_or_default();
        let id = text
            .lines()
            .find_map(|line| line.strip_prefix("# id: "))
            .unwrap_or("?")
            .trim()
            .to_string();
        for line in text.lines().take_while(|line| line.starts_with('#')) {
            if let Some(names) = line.strip_prefix("# covers: ") {
                for name in names.split_whitespace() {
                    covers.entry(name.to_string()).or_default().push(id.clone());
                }
            }
        }
    }
    covers
}
