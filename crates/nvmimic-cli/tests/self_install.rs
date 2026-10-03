//! `nvmimic self install` and `self uninstall`, run as a user would, in
//! throwaway homes.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

struct Home {
    root: PathBuf,
}

impl Home {
    fn new() -> Home {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("nvmimic-self-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        Home { root }
    }

    fn nvm_dir(&self) -> PathBuf {
        self.root.join(".nvm")
    }

    fn nvmimic(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_nvmimic"))
            .args(args)
            .env_clear()
            .env("HOME", &self.root)
            .env("PATH", "/usr/bin:/bin")
            .env("HOMEBREW_PREFIX", self.root.join("no-homebrew"))
            .output()
            .unwrap()
    }

    /// A shell that sources the installed nvm.sh and runs `body`.
    fn shell(&self, body: &str) -> Output {
        Command::new("/bin/bash")
            .arg("-c")
            .arg(format!(". \"$NVM_DIR/nvm.sh\" --no-use\n{body}"))
            .env_clear()
            .env("HOME", &self.root)
            .env("NVM_DIR", self.nvm_dir())
            .env("PATH", "/usr/bin:/bin")
            .output()
            .unwrap()
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn reference() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/reference/nvm-v0.40.8")
}

/// A `$NVM_DIR` the way nvm's installer leaves it, with one installed version.
fn real_nvm(home: &Home) {
    let dir = home.nvm_dir();
    fs::create_dir_all(dir.join("versions/node/v22.1.0/bin")).unwrap();
    fs::create_dir_all(dir.join("alias")).unwrap();
    fs::write(dir.join("alias/default"), "22\n").unwrap();
    for file in ["nvm.sh", "nvm-exec", "bash_completion"] {
        fs::copy(reference().join(file), dir.join(file)).unwrap();
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn fresh_install_works_and_uninstall_removes_it() {
    let home = Home::new();
    let out = home.nvmimic(&["self", "install", "--yes"]);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    assert!(text(&out.stdout).contains("add these lines"));

    let nvm = home.shell("nvm --version; command -v nvm");
    assert_eq!(text(&nvm.stdout), "0.40.8\nnvm\n", "{}", text(&nvm.stderr));

    let out = home.nvmimic(&["self", "uninstall", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!home.nvm_dir().join(".nvmimic").exists());
    assert!(!home.nvm_dir().join("nvm.sh").exists());
}

#[test]
fn takeover_keeps_versions_and_uninstall_restores_nvm() {
    let home = Home::new();
    real_nvm(&home);
    let out = home.nvmimic(&["self", "install", "--yes"]);
    let stdout = text(&out.stdout);
    assert!(out.status.success(), "{stdout}{}", text(&out.stderr));
    assert!(stdout.contains("drop-in replacement"));
    assert!(
        home.root.join(".nvm.backup-1/nvm.sh").is_file(),
        "full backup made"
    );

    let dir = home.nvm_dir();
    assert!(dir.join("versions/node/v22.1.0/bin").is_dir());
    assert_eq!(
        fs::read_to_string(dir.join("alias/default")).unwrap(),
        "22\n"
    );
    assert_eq!(
        fs::read(dir.join(".nvmimic/backup/nvm.sh")).unwrap(),
        fs::read(reference().join("nvm.sh")).unwrap()
    );
    let nvm = home.shell("nvm alias default");
    assert_eq!(
        text(&nvm.stdout),
        "default -> 22 (-> v22.1.0 *)\n",
        "{}",
        text(&nvm.stderr)
    );

    let out = home.nvmimic(&["self", "uninstall", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    for file in ["nvm.sh", "nvm-exec", "bash_completion"] {
        assert_eq!(
            fs::read(dir.join(file)).unwrap(),
            fs::read(reference().join(file)).unwrap()
        );
    }
    assert!(dir.join("versions/node/v22.1.0/bin").is_dir());
}

#[test]
fn dry_run_changes_nothing() {
    let home = Home::new();
    real_nvm(&home);
    let out = home.nvmimic(&["self", "install", "--yes", "--dry-run"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("Would install nvmimic"));
    assert!(!home.nvm_dir().join(".nvmimic").exists());
    assert!(!home.root.join(".nvm.backup-1").exists());
    assert_eq!(
        fs::read(home.nvm_dir().join("nvm.sh")).unwrap(),
        fs::read(reference().join("nvm.sh")).unwrap()
    );
}

#[test]
fn git_installs_get_a_warning() {
    let home = Home::new();
    real_nvm(&home);
    fs::create_dir_all(home.nvm_dir().join(".git")).unwrap();
    let out = home.nvmimic(&["self", "install", "--yes"]);
    assert!(
        text(&out.stdout).contains("installed with git"),
        "{}",
        text(&out.stdout)
    );
}

#[test]
fn without_a_terminal_questions_need_yes() {
    let home = Home::new();
    let out = home.nvmimic(&["self", "install"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("--yes"));
}

#[test]
fn homebrew_nvm_is_uninstalled_first() {
    let home = Home::new();
    let prefix = home.root.join("brew");
    fs::create_dir_all(prefix.join("opt/nvm")).unwrap();
    fs::create_dir_all(prefix.join("bin")).unwrap();
    let log = home.root.join("brew.log");
    let brew = prefix.join("bin/brew");
    fs::write(
        &brew,
        format!("#!/bin/sh\necho \"$@\" >> '{}'\n", log.display()),
    )
    .unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&brew, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fs::write(
        home.root.join(".zshrc"),
        "\\. \"/opt/homebrew/opt/nvm/nvm.sh\"\n",
    )
    .unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_nvmimic"))
        .args(["self", "install", "--yes"])
        .env_clear()
        .env("HOME", &home.root)
        .env("PATH", "/usr/bin:/bin")
        .env("HOMEBREW_PREFIX", &prefix)
        .output()
        .unwrap();
    let stdout = text(&out.stdout);
    assert!(out.status.success(), "{stdout}{}", text(&out.stderr));
    assert!(stdout.contains("Homebrew's nvm is installed"));
    assert!(stdout.contains(".zshrc loads Homebrew's nvm"));
    assert_eq!(fs::read_to_string(&log).unwrap(), "uninstall nvm\n");
}
