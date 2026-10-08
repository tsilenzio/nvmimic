//! `eval "$(nvmimic init <shell>)"` written straight into a startup file,
//! with no `nvm.sh` in between, under each shell there is.
//!
//! The tests tell one start of the program from two by taking the program
//! away after the first. A shell that still ends up on the default version
//! didn't need a second start.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Each shell with the name `init` takes for it.
const SHELLS: &[(&str, &str)] = &[
    ("/bin/bash", "bash"),
    ("/bin/zsh", "zsh"),
    ("/bin/sh", "sh"),
    ("/bin/dash", "dash"),
];

fn shells() -> impl Iterator<Item = (&'static str, &'static str)> {
    SHELLS
        .iter()
        .copied()
        .filter(|(path, _)| Path::new(path).exists())
}

struct Home {
    root: PathBuf,
    /// `XDG_CONFIG_HOME` for the shells this home starts, if any.
    config: Option<PathBuf>,
}

impl Home {
    fn new() -> Home {
        sweep_stale();
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("nvmimic-direct-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("bin")).unwrap();
        // A link and not a copy: macOS scans every new program it runs.
        std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_nvmimic"), root.join("bin/nvmimic"))
            .unwrap();
        Home { root, config: None }
    }

    /// The nvmimic program as the startup line names it.
    fn program(&self) -> PathBuf {
        self.root.join("bin/nvmimic")
    }

    /// An nvm folder with one version in it, which is the default.
    fn nvm_dir(&self, name: &str, version: &str) -> PathBuf {
        let dir = self.root.join(name);
        let bin = dir.join(format!("versions/node/{version}/bin"));
        fs::create_dir_all(&bin).unwrap();
        fs::create_dir_all(dir.join("alias")).unwrap();
        fs::write(dir.join("alias/default"), format!("{version}\n")).unwrap();
        fs::write(bin.join("node"), format!("#!/bin/sh\necho {version}\n")).unwrap();
        fs::set_permissions(bin.join("node"), fs::Permissions::from_mode(0o755)).unwrap();
        dir
    }

    /// Runs the startup line in `shell` and prints `$NVM_DIR` and `$PATH`
    /// afterwards. With `keep` false the program is gone after its first
    /// start.
    fn load(&self, shell: &str, init: &str, nvm_dir: Option<&Path>, keep: bool) -> Loaded {
        let program = self.program();
        let away = self.root.join("bin/away");
        let hide = if keep {
            String::new()
        } else {
            format!("mv '{}' '{}'", program.display(), away.display())
        };
        let script = format!(
            "set -eu\ntext=\"$('{}' {init})\"\n{hide}\neval \"$text\"\nprintf '%s\\n' \"$NVM_DIR\" \"$PATH\"\n",
            program.display()
        );
        let mut command = Command::new(shell);
        command
            .arg("-c")
            .arg(script)
            .env_clear()
            .env("HOME", &self.root)
            .env("PATH", "/usr/bin:/bin")
            .env("TERM", "xterm")
            // A login session exports SHELL. bash sets it for itself when
            // it's missing, without exporting it, and that alone costs a
            // second start.
            .env("SHELL", "/bin/zsh")
            .current_dir(&self.root);
        if let Some(nvm_dir) = nvm_dir {
            command.env("NVM_DIR", nvm_dir);
        }
        if let Some(config) = &self.config {
            command.env("XDG_CONFIG_HOME", config);
        }
        let output = command.output().unwrap();
        if !keep {
            let _ = fs::rename(&away, &program);
        }
        Loaded::of(output)
    }
}

/// What a shell held after the startup line.
struct Loaded {
    ok: bool,
    nvm_dir: String,
    path: String,
    stderr: String,
}

impl Loaded {
    fn of(output: Output) -> Loaded {
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let mut lines = stdout.lines();
        Loaded {
            ok: output.status.success(),
            nvm_dir: lines.next().unwrap_or("").to_string(),
            path: lines.next().unwrap_or("").to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// Whether the shell ended up on the version in `nvm_dir`.
    fn uses(&self, nvm_dir: &Path, version: &str) -> bool {
        let bin = format!("{}/versions/node/{version}/bin:", nvm_dir.display());
        self.ok && self.path.starts_with(&bin) && self.stderr.is_empty()
    }
}

/// Removes homes that earlier runs left behind, once they are half an hour
/// old. Nothing is deleted during a run, since each shell runs a `node`
/// stand-in that macOS may still be scanning.
fn sweep_stale() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let Ok(entries) = fs::read_dir(std::env::temp_dir()) else {
            return;
        };
        for entry in entries.flatten() {
            let ours = entry
                .file_name()
                .to_string_lossy()
                .starts_with("nvmimic-direct-");
            let old = entry
                .metadata()
                .and_then(|meta| meta.modified())
                .ok()
                .and_then(|time| time.elapsed().ok())
                .is_some_and(|age| age.as_secs() > 1800);
            if ours && old {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
    });
}

#[test]
fn the_named_shell_loads_with_one_start() {
    for (shell, name) in shells() {
        let home = Home::new();
        let nvm_dir = home.nvm_dir(".nvm", "v22.1.0");
        let loaded = home.load(shell, &format!("init {name}"), Some(&nvm_dir), false);
        assert!(
            loaded.uses(&nvm_dir, "v22.1.0"),
            "{shell} with `init {name}`: PATH {} stderr {}",
            loaded.path,
            loaded.stderr
        );
    }
}

#[test]
fn a_wrong_name_costs_a_second_start_and_says_nothing() {
    for (shell, name) in shells() {
        // Any name that isn't this shell's. `sh` fits bash in POSIX mode
        // and dash, so those two get `zsh`.
        let wrong = if name == "zsh" { "bash" } else { "zsh" };
        let home = Home::new();
        let nvm_dir = home.nvm_dir(".nvm", "v22.1.0");
        let init = format!("init {wrong}");

        let loaded = home.load(shell, &init, Some(&nvm_dir), true);
        assert!(
            loaded.uses(&nvm_dir, "v22.1.0"),
            "{shell} with `{init}`: PATH {} stderr {}",
            loaded.path,
            loaded.stderr
        );

        // Without the program the second start has nothing to run, which
        // shows that this shell asked for one.
        let stranded = home.load(shell, &init, Some(&nvm_dir), false);
        assert!(
            !stranded.ok && !stranded.path.contains("v22.1.0"),
            "{shell} with `{init}` switched without a second start"
        );
    }
}

#[test]
fn no_name_always_takes_the_second_start() {
    for (shell, _) in shells() {
        let home = Home::new();
        let nvm_dir = home.nvm_dir(".nvm", "v22.1.0");
        assert!(
            home.load(shell, "init", Some(&nvm_dir), true)
                .uses(&nvm_dir, "v22.1.0"),
            "{shell}"
        );
        assert!(
            !home.load(shell, "init", Some(&nvm_dir), false).ok,
            "{shell}"
        );
    }
}

#[test]
fn without_nvm_dir_the_folder_is_home_nvm() {
    for (shell, name) in shells() {
        let home = Home::new();
        let nvm_dir = home.nvm_dir(".nvm", "v22.1.0");
        let loaded = home.load(shell, &format!("init {name}"), None, false);
        assert_eq!(loaded.nvm_dir, nvm_dir.display().to_string(), "{shell}");
        assert!(
            loaded.uses(&nvm_dir, "v22.1.0"),
            "{shell}: {}",
            loaded.stderr
        );
    }
}

#[test]
fn without_nvm_dir_the_folder_is_the_one_nvms_installer_would_pick() {
    // nvm's installer uses `$XDG_CONFIG_HOME/nvm` when that variable is
    // set, and `~/.nvm` only when it isn't.
    for (shell, name) in shells() {
        let mut home = Home::new();
        home.nvm_dir(".nvm", "v20.3.0");
        let nvm_dir = home.nvm_dir("config/nvm", "v22.1.0");
        home.config = Some(home.root.join("config"));
        let loaded = home.load(shell, &format!("init {name}"), None, false);
        assert_eq!(loaded.nvm_dir, nvm_dir.display().to_string(), "{shell}");
        assert!(
            loaded.uses(&nvm_dir, "v22.1.0"),
            "{shell}: {}",
            loaded.stderr
        );
    }
}

#[test]
fn without_nvm_dir_the_folder_nvmimic_was_set_up_for_comes_first() {
    let home = Home::new();
    home.nvm_dir(".nvm", "v22.1.0");
    let elsewhere = home.nvm_dir("elsewhere", "v20.3.0");
    let installed = Command::new(home.program())
        .args(["self", "install", "--force"])
        .env_clear()
        .env("HOME", &home.root)
        .env("PATH", "/usr/bin:/bin")
        .env("NVM_DIR", &elsewhere)
        .env("HOMEBREW_PREFIX", home.root.join("no-homebrew"))
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(
        installed.status.success(),
        "{}",
        String::from_utf8_lossy(&installed.stderr)
    );

    for (shell, name) in shells() {
        for init in [format!("init {name}"), "init".to_string()] {
            let loaded = home.load(shell, &init, None, true);
            assert_eq!(
                loaded.nvm_dir,
                elsewhere.display().to_string(),
                "{shell} {init}"
            );
            assert!(
                loaded.uses(&elsewhere, "v20.3.0"),
                "{shell} {init}: {}",
                loaded.stderr
            );
        }
    }
}

#[test]
fn a_shell_init_does_not_know_is_refused() {
    let home = Home::new();
    let output = Command::new(home.program())
        .args(["init", "fish"])
        .env_clear()
        .env("HOME", &home.root)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("bash, zsh, sh, dash"));
}
