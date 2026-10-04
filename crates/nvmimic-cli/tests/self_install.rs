//! `nvmimic self install` and `self uninstall`, run as a user would, in
//! throwaway homes.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

struct Home {
    root: PathBuf,
}

impl Home {
    fn new() -> Home {
        sweep_stale();
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
        self.nvmimic_with(args, &[])
    }

    fn nvmimic_with(&self, args: &[&str], vars: &[(&str, &Path)]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_nvmimic"))
            .args(args)
            .env_clear()
            .env("HOME", &self.root)
            .env("PATH", "/usr/bin:/bin")
            .env("HOMEBREW_PREFIX", self.root.join("no-homebrew"))
            .envs(vars.iter().copied())
            .output()
            .unwrap()
    }

    /// The nvmimic program where `self install` puts it.
    fn command(&self) -> PathBuf {
        self.root.join(".local/bin/nvmimic")
    }

    /// nvmimic's own folders: installation files, install record, and
    /// remembered answers.
    fn own(&self) -> [PathBuf; 3] {
        let cache = if cfg!(target_os = "macos") {
            "Library/Caches/nvmimic"
        } else {
            ".cache/nvmimic"
        };
        [
            self.root.join(".local/share/nvmimic"),
            self.root.join(".local/state/nvmimic"),
            self.root.join(cache),
        ]
    }

    fn state(&self) -> PathBuf {
        self.root.join(".local/state/nvmimic")
    }

    /// Whether anything of nvmimic's own is left.
    fn has_own_files(&self) -> bool {
        self.own().iter().any(|dir| dir.exists()) || self.command().exists()
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

// A `Home` is left behind on purpose and swept by a later run. Each test
// copies the nvmimic program into its home and runs it, and macOS scans a
// new program the first time it runs. Deleting the copy while that scan may
// still be queued is part of what crashed the scanning service in an earlier run.

/// Removes homes that earlier runs left behind, once they are half an hour old.
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
                .starts_with("nvmimic-self-");
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

fn reference() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/reference/nvm-v0.40.8")
}

/// A `$NVM_DIR` the way nvm's installer leaves it, with one installed version.
fn real_nvm(home: &Home) {
    let dir = home.nvm_dir();
    fs::create_dir_all(dir.join("versions/node/v22.1.0/bin")).unwrap();
    fs::create_dir_all(dir.join("alias")).unwrap();
    fs::write(dir.join("alias/default"), "22\n").unwrap();
    for file in SCRIPTS {
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
    assert!(!home.has_own_files());
    assert!(!home.nvm_dir().join("nvm.sh").exists());
}

#[test]
fn nothing_of_nvmimics_own_goes_into_nvm_dir() {
    let home = Home::new();
    real_nvm(&home);
    let node = home.nvm_dir().join("versions/node/v22.1.0/bin/node");
    fs::write(&node, "#!/bin/sh\necho v22.1.0\n").unwrap();
    fs::set_permissions(&node, fs::Permissions::from_mode(0o755)).unwrap();
    let before: Vec<PathBuf> = snapshot(&home.nvm_dir())
        .into_iter()
        .map(|(path, _)| path)
        .collect();
    let out = home.nvmimic(&["self", "install", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    // One `nvm use` makes nvmimic remember node's and npm's answers.
    let _ = home.shell("nvm use 22");
    let after: Vec<PathBuf> = snapshot(&home.nvm_dir())
        .into_iter()
        .map(|(path, _)| path)
        .collect();
    assert_eq!(after, before);
    for dir in home.own() {
        assert!(dir.is_dir(), "{} is missing", dir.display());
    }
    let [_, _, cache] = home.own();
    assert!(
        fs::read_dir(cache.join("node-version")).unwrap().count() == 1,
        "node's answer was remembered outside $NVM_DIR"
    );
}

#[test]
fn own_folders_follow_their_variables() {
    let home = Home::new();
    let elsewhere = home.root.join("elsewhere");
    let vars = [
        ("NVMIMIC_INSTALL_PATH", elsewhere.join("tools/nvmimic")),
        ("NVMIMIC_DATA_DIR", elsewhere.join("data")),
        ("NVMIMIC_STATE_DIR", elsewhere.join("state")),
        ("NVMIMIC_CACHE_DIR", elsewhere.join("cache")),
    ];
    let vars: Vec<(&str, &Path)> = vars.iter().map(|(k, v)| (*k, v.as_path())).collect();
    // Something of the user's own in the data folder has to survive.
    fs::create_dir_all(elsewhere.join("data")).unwrap();
    fs::write(elsewhere.join("data/mine"), "mine\n").unwrap();

    let out = home.nvmimic_with(&["self", "install", "--yes"], &vars);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(elsewhere.join("tools/nvmimic").is_file());
    assert!(elsewhere.join("data/nvm-functions.sh").is_file());
    assert!(elsewhere.join("state/manifest").is_file());
    assert!(!home.has_own_files());
    let nvm = home.shell("nvm --version");
    assert_eq!(text(&nvm.stdout), "0.40.8\n", "{}", text(&nvm.stderr));

    let out = home.nvmimic_with(&["self", "uninstall", "--yes"], &vars);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!elsewhere.join("tools/nvmimic").exists());
    assert!(!elsewhere.join("state").exists());
    assert_eq!(
        fs::read_to_string(elsewhere.join("data/mine")).unwrap(),
        "mine\n"
    );
    assert!(!elsewhere.join("data/nvm-functions.sh").exists());
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
        fs::read(home.state().join("backup/nvm.sh")).unwrap(),
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
    assert!(!home.has_own_files());
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

/// Runs git in a directory with a throwaway identity and returns its stdout.
fn git(home: &Home, dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=test", "-c", "user.email=test@example.com"])
        .args(args)
        .env_clear()
        .env("HOME", &home.root)
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {}", text(&out.stderr));
    text(&out.stdout)
}

/// Turns the `$NVM_DIR` from `real_nvm` into a git checkout, the way nvm's
/// installer leaves one.
fn make_git_checkout(home: &Home) {
    let dir = home.nvm_dir();
    // nvm's own .gitignore covers what nvm writes into its checkout.
    fs::write(dir.join(".gitignore"), "alias\nversions\n.cache\n").unwrap();
    git(home, &dir, &["init", "-q"]);
    git(
        home,
        &dir,
        &["add", "nvm.sh", "nvm-exec", "bash_completion", ".gitignore"],
    );
    git(home, &dir, &["commit", "-q", "-m", "nvm"]);
}

#[test]
fn git_clone_takeover_shows_only_the_swapped_scripts_as_modified() {
    let home = Home::new();
    real_nvm(&home);
    make_git_checkout(&home);
    let dir = home.nvm_dir();
    let status = || git(&home, &dir, &["status", "--porcelain"]);
    let before = status();
    assert_eq!(before, "");

    let out = home.nvmimic(&["self", "install", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let installed = status();
    for line in installed.lines() {
        assert!(
            matches!(line, " M nvm.sh" | " M nvm-exec" | " M bash_completion"),
            "unexpected line in git status: {line}"
        );
    }
    assert!(installed.contains(" M nvm.sh"), "{installed}");
    // The checkout itself is left alone: nothing is hidden from git.
    assert!(
        git(&home, &dir, &["ls-files", "-v"])
            .lines()
            .all(|line| line.starts_with('H'))
    );

    let out = home.nvmimic(&["self", "uninstall", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(status(), before, "same as before the takeover");
}

#[test]
fn install_puts_the_program_in_local_bin() {
    let home = Home::new();
    let command = home.command();
    let out = home.nvmimic(&["self", "install", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(fs::symlink_metadata(&command).unwrap().is_file());
    assert!(text(&out.stdout).contains("isn't on your PATH yet"));
    // Nothing named nvm goes on PATH: real nvm is only a shell function.
    assert!(!home.root.join(".local/bin/nvm").exists());

    // The installed program can install again, and stays the same file.
    let again = Command::new(&command)
        .args(["self", "install", "--yes"])
        .env_clear()
        .env("HOME", &home.root)
        .env("PATH", "/usr/bin:/bin")
        .env("HOMEBREW_PREFIX", home.root.join("no-homebrew"))
        .output()
        .unwrap();
    assert!(again.status.success(), "{}", text(&again.stderr));
    assert!(command.is_file());

    let out = home.nvmimic(&["self", "uninstall", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(fs::symlink_metadata(&command).is_err());
}

#[test]
fn a_nvmimic_command_that_is_someone_elses_is_left_alone() {
    let home = Home::new();
    let command = home.command();
    fs::create_dir_all(command.parent().unwrap()).unwrap();
    fs::write(&command, "mine\n").unwrap();

    let out = home.nvmimic(&["self", "install", "--yes"]);
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("is left alone"),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(fs::read_to_string(&command).unwrap(), "mine\n");
    assert!(!home.nvm_dir().join("nvm.sh").exists());

    // Naming another place for the program gets past it.
    let elsewhere = home.root.join("tools/nvmimic");
    let vars = [("NVMIMIC_INSTALL_PATH", elsewhere.as_path())];
    let out = home.nvmimic_with(&["self", "install", "--yes"], &vars);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(elsewhere.is_file());
    let out = home.nvmimic_with(&["self", "uninstall", "--yes"], &vars);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!elsewhere.exists());
    assert_eq!(fs::read_to_string(&command).unwrap(), "mine\n");
}

#[test]
fn a_second_nvm_dir_needs_an_uninstall_first() {
    let home = Home::new();
    let out = home.nvmimic(&["self", "install", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let other = home.root.join("other-nvm");
    let out = home.nvmimic_with(
        &["self", "install", "--yes"],
        &[("NVM_DIR", other.as_path())],
    );
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("already installed for"),
        "{}",
        text(&out.stderr)
    );
    assert!(!other.exists());
}

const SCRIPTS: [&str; 3] = ["nvm.sh", "nvm-exec", "bash_completion"];

/// Runs nvm's own install script against the home, offline. `method` is
/// nvm's `METHOD`: `git` fetches from a local repository holding the
/// reference files, and `script` downloads them through a stand-in `curl`.
fn nvm_installer(home: &Home, method: &str) -> Output {
    let origin = home.root.join("origin");
    if !origin.exists() {
        fs::create_dir_all(&origin).unwrap();
        for file in SCRIPTS {
            fs::copy(reference().join(file), origin.join(file)).unwrap();
        }
        fs::write(origin.join(".gitignore"), "alias\nversions\n.cache\n").unwrap();
        git(home, &origin, &["init", "-q"]);
        git(home, &origin, &["add", "."]);
        git(home, &origin, &["commit", "-q", "-m", "nvm"]);
        git(home, &origin, &["tag", "v0.40.8"]);

        let shim = home.root.join("shim");
        fs::create_dir_all(&shim).unwrap();
        let curl = shim.join("curl");
        fs::write(
            &curl,
            format!(
                "#!/bin/sh\n\
                 # Stands in for curl: copies the reference file the URL names.\n\
                 while [ $# -gt 0 ]; do\n\
                 \x20 case \"$1\" in\n\
                 \x20   -o) out=\"$2\"; shift ;;\n\
                 \x20   -*) ;;\n\
                 \x20   *) url=\"$1\" ;;\n\
                 \x20 esac\n\
                 \x20 shift\n\
                 done\n\
                 cp '{}'/\"${{url##*/}}\" \"$out\"\n",
                reference().display()
            ),
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&curl, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut installer = Command::new("/bin/bash");
    installer
        .arg(reference().join("install.sh"))
        .env_clear()
        .env("HOME", &home.root)
        .env("NVM_DIR", home.nvm_dir())
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", home.root.join("shim").display()),
        )
        .env("METHOD", method)
        .env("NVM_INSTALL_VERSION", "v0.40.8")
        .env("PROFILE", "/dev/null");
    if method == "git" {
        installer.env("NVM_SOURCE", format!("file://{}", origin.display()));
    }
    let out = installer.output().unwrap();
    assert!(
        out.status.success(),
        "nvm's installer ({method}): {}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    out
}

/// Every file under `dir` with its bytes, symlinks as their targets.
fn snapshot(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in fs::read_dir(&next).unwrap() {
            let path = entry.unwrap().path();
            let meta = fs::symlink_metadata(&path).unwrap();
            if meta.is_symlink() {
                let target = fs::read_link(&path).unwrap();
                found.push((path, target.to_string_lossy().into_owned().into_bytes()));
            } else if meta.is_dir() {
                pending.push(path);
            } else {
                let bytes = fs::read(&path).unwrap();
                found.push((path, bytes));
            }
        }
    }
    found.sort();
    found
}

/// nvm gets installed by its own installer, nvmimic takes over, nvm's
/// installer is run again, and nvmimic takes over again.
fn installer_rerun(method: &str) {
    let home = Home::new();
    let dir = home.nvm_dir();
    // Everything of nvmimic's own in this home sits under `.local`.
    let own = home.root.join(".local");
    let is_git = method == "git";
    nvm_installer(&home, method);
    assert_eq!(dir.join(".git").exists(), is_git);
    fs::create_dir_all(dir.join("versions/node/v22.1.0/bin")).unwrap();
    fs::create_dir_all(dir.join("alias")).unwrap();
    fs::write(dir.join("alias/default"), "22\n").unwrap();

    let out = home.nvmimic(&["self", "install", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let manifest = fs::read_to_string(home.state().join("manifest")).unwrap();
    for file in SCRIPTS {
        assert!(manifest.contains(&format!("backup {file}\n")), "{manifest}");
    }
    let ours: Vec<Vec<u8>> = SCRIPTS
        .iter()
        .map(|file| fs::read(dir.join(file)).unwrap())
        .collect();
    let own_before = snapshot(&own);
    let rest_before: Vec<_> = snapshot(&dir)
        .into_iter()
        .filter(|(path, _)| {
            !path.starts_with(dir.join(".git"))
                && !SCRIPTS.iter().any(|file| *path == dir.join(file))
        })
        .collect();

    // nvm's installer writes its three scripts back and touches nothing else.
    nvm_installer(&home, method);
    for file in SCRIPTS {
        assert_eq!(
            fs::read(dir.join(file)).unwrap(),
            fs::read(reference().join(file)).unwrap(),
            "{file} is nvm's again"
        );
    }
    assert_eq!(
        snapshot(&own),
        own_before,
        "nvmimic's own files are untouched"
    );
    let rest_after: Vec<_> = snapshot(&dir)
        .into_iter()
        .filter(|(path, _)| {
            !path.starts_with(dir.join(".git"))
                && !SCRIPTS.iter().any(|file| *path == dir.join(file))
        })
        .collect();
    assert_eq!(rest_after, rest_before, "nothing else changed");
    if is_git {
        assert_eq!(git(&home, &dir, &["status", "--porcelain"]), "");
    }

    // The command left in ~/.local/bin takes over again.
    let out = Command::new(home.root.join(".local/bin/nvmimic"))
        .args(["self", "install", "--yes"])
        .env_clear()
        .env("HOME", &home.root)
        .env("PATH", "/usr/bin:/bin")
        .env("HOMEBREW_PREFIX", home.root.join("no-homebrew"))
        .output()
        .unwrap();
    let stdout = text(&out.stdout);
    assert!(out.status.success(), "{stdout}{}", text(&out.stderr));
    assert!(stdout.contains("Taking over again"), "{stdout}");
    assert!(
        !stdout.contains("Copy all of"),
        "no second full backup is offered: {stdout}"
    );
    assert!(!home.root.join(".nvm.backup-2").exists());
    for (file, bytes) in SCRIPTS.iter().zip(&ours) {
        assert_eq!(&fs::read(dir.join(file)).unwrap(), bytes, "{file} is ours");
        assert_eq!(
            fs::read(home.state().join("backup").join(file)).unwrap(),
            fs::read(reference().join(file)).unwrap(),
            "{file} backup is nvm's"
        );
    }
    assert_eq!(snapshot(&own), own_before, "manifest, backups and binary");
    let nvm = home.shell("nvm alias default");
    assert_eq!(
        text(&nvm.stdout),
        "default -> 22 (-> v22.1.0 *)\n",
        "{}",
        text(&nvm.stderr)
    );

    let out = home.nvmimic(&["self", "uninstall", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    for file in SCRIPTS {
        assert_eq!(
            fs::read(dir.join(file)).unwrap(),
            fs::read(reference().join(file)).unwrap()
        );
    }
    assert!(!home.has_own_files());
    if is_git {
        assert_eq!(git(&home, &dir, &["status", "--porcelain"]), "");
    }
}

#[test]
fn nvm_installer_rerun_with_git_then_retakeover() {
    installer_rerun("git");
}

#[test]
fn nvm_installer_rerun_as_script_then_retakeover() {
    installer_rerun("script");
}

#[test]
fn nvm_installed_over_a_fresh_nvmimic_survives_uninstall() {
    let home = Home::new();
    let dir = home.nvm_dir();
    let out = home.nvmimic(&["self", "install", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    nvm_installer(&home, "script");
    let out = home.nvmimic(&["self", "install", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let out = home.nvmimic(&["self", "uninstall", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    for file in SCRIPTS {
        assert_eq!(
            fs::read(dir.join(file)).unwrap(),
            fs::read(reference().join(file)).unwrap(),
            "{file} is nvm's and stays"
        );
    }
}
