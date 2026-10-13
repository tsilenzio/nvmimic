//! A second program through the harness: `shelfy`, the example program of
//! the nvmimic library (`crates/nvmimic/examples/shelfy.rs`). It keeps Node
//! versions on a shelf outside `$NVM_DIR`, so it differs from official nvm
//! on purpose in where files are, and nowhere else. This is what a program
//! built on the library writes to be compared the way nvmimic is.

use nvmimic_parity::cases::{self, Plan};
use nvmimic_parity::harness::Impl;
use nvmimic_parity::subject::{Expected, Subject};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

struct Shelfy;

/// Where shelfy keeps its own files in a sandbox: beside the home folder,
/// each with the setting that moves it there.
fn own_folders(side: &Path) -> Vec<(String, String)> {
    let own = side.join("own");
    [
        ("SHELFY_INSTALL_PATH", own.join("bin/shelfy")),
        ("SHELFY_STATE_DIR", own.join("state")),
        ("SHELFY_CACHE_DIR", own.join("cache")),
        ("SHELFY_DATA_DIR", own.join("data")),
    ]
    .into_iter()
    .map(|(name, path)| (name.to_string(), path.display().to_string()))
    .collect()
}

impl Subject for Shelfy {
    fn name(&self) -> &str {
        "shelfy"
    }

    /// Runs shelfy's real installer. The program is a link to the one
    /// cargo built, and `SHELFY_INSTALL_PATH` names that link, so the
    /// installer keeps it where it is and copies nothing.
    fn install(&self, nvm_dir: &Path, side: &Path, _count_launches: bool) {
        let program = side.join("own/bin/shelfy");
        fs::create_dir_all(program.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_shelfy"), &program).unwrap();
        let installed = Command::new(&program)
            .args(["self", "install", "--force"])
            .env_clear()
            .env("HOME", side.join("home"))
            .env("NVM_DIR", nvm_dir)
            .env("PATH", "/usr/bin:/bin")
            .env("HOMEBREW_PREFIX", side.join("no-homebrew"))
            .envs(own_folders(side))
            // The record shared with sister tools, which only the `self`
            // commands read and no program's own setting moves.
            .env("NVMIMIC_SHARED_DIR", side.join("own/shared"))
            .output()
            .unwrap();
        assert!(
            installed.status.success(),
            "shelfy self install: {}",
            String::from_utf8_lossy(&installed.stderr)
        );
    }

    /// The program finds its record of the install through these.
    fn env(&self, side: &Path) -> Vec<(String, String)> {
        own_folders(side)
    }

    fn own_functions(&self) -> Vec<String> {
        ["__shelfy_handoff", "_shelfy_complete"]
            .map(str::to_string)
            .to_vec()
    }

    /// The shelf names a version's folder without the `v`.
    fn normalize(&self, text: String) -> String {
        text.replace("<home>/shelf/", "<install>/v")
    }

    /// Downloads and locks are the shelf's own business.
    fn ignores_file(&self, relative: &str) -> bool {
        Path::new(relative).starts_with("shelf-cache")
    }

    fn expected(&self) -> &Expected {
        static EXPECTED: OnceLock<Expected> = OnceLock::new();
        EXPECTED.get_or_init(|| {
            Expected::parse(
                "# Versions and aliases are on the shelf, not in the nvm folder.\n\
                 * * fs:home/shelf*\n\
                 * * fs:home/.nvm/versions*\n\
                 * * fs:home/.nvm/alias*\n\
                 * * fs:home/.nvm/.cache*\n\
                 # What shelfy sets for itself.\n\
                 * * var:SHELFY_*\n\
                 * * var:SHELF_NODE_VERSION\n\
                 * * var:SHELF_SWITCH\n",
            )
        })
    }
}

/// A few cases that install a version, switch to it and ask where it is.
/// The same case files nvmimic runs, against the same recordings of
/// official nvm.
const CASES: [&str; 4] = ["VER-01", "VER-02", "LST-01", "HLP-01a"];

#[test]
fn a_second_program_is_compared_like_nvmimic() {
    let all = cases::load_all();
    let picked: Vec<cases::Case> = all
        .into_iter()
        .filter(|case| CASES.contains(&case.id.as_str()))
        .collect();
    assert!(!picked.is_empty(), "none of {CASES:?} is among the cases");

    let plan = Plan::everyday_for(Impl::Subject(&Shelfy), true);
    let results = cases::run_all(&picked, plan).unwrap_or_else(|reason| {
        panic!("the run was stopped to protect the machine: {reason}");
    });
    assert!(results.len() >= picked.len());
    let known = std::collections::BTreeSet::new();
    if let Err(message) = cases::judge(&results, &known, "no list") {
        panic!("{message}");
    }
}
