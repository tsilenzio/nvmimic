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
            // No terminal, whoever runs the tests.
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap()
    }

    /// The nvmimic program where `self install` puts it.
    fn command(&self) -> PathBuf {
        self.root.join(".local/bin/nvmimic")
    }

    /// nvmimic's own folders: installation files, install record, and
    /// remembered answers. The last one is the record it shares with
    /// sister tools, which goes when no program is left in it.
    fn own(&self) -> [PathBuf; 4] {
        let cache = if cfg!(target_os = "macos") {
            "Library/Caches/nvmimic"
        } else {
            ".cache/nvmimic"
        };
        [
            self.root.join(".local/share/nvmimic"),
            self.root.join(".local/state/nvmimic"),
            self.root.join(cache),
            self.shared(),
        ]
    }

    fn state(&self) -> PathBuf {
        self.root.join(".local/state/nvmimic")
    }

    /// The record shared with sister tools, and official nvm's scripts.
    fn shared(&self) -> PathBuf {
        self.root.join(".local/state/nvmimic-shared")
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
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    assert!(text(&out.stdout).contains("add these lines"));

    let nvm = home.shell("nvm --version; command -v nvm");
    assert_eq!(text(&nvm.stdout), "0.40.8\nnvm\n", "{}", text(&nvm.stderr));

    let out = home.nvmimic(&["self", "uninstall", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!home.has_own_files());
    assert!(!home.nvm_dir().join("nvm.sh").exists());
}

#[test]
fn a_shell_holding_another_version_gets_new_functions() {
    let home = Home::new();
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    // The shell's functions are rewritten to say they came from another
    // version, which is what a shell left open across an update holds.
    let nvm = home.shell(
        r#"eval "$(declare -f __nvmimic_handoff | sed "s/__NVMIMIC_CODE='[^']*'/__NVMIMIC_CODE='0.0.0-old'/")"
declare -f __nvmimic_handoff | grep -c 0.0.0-old
nvm --version
declare -f __nvmimic_handoff | grep -c 0.0.0-old
nvm --version"#,
    );
    assert_eq!(
        text(&nvm.stdout),
        "1\n0.40.8\n0\n0.40.8\n",
        "{}",
        text(&nvm.stderr)
    );
}

#[test]
fn a_missing_program_gets_one_message() {
    let home = Home::new();
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    fs::rename(home.command(), home.root.join("moved-away")).unwrap();
    let nvm = home.shell("echo \"status $?\"; command -v nvm");
    assert_eq!(text(&nvm.stdout), "status 127\n");
    let message = text(&nvm.stderr);
    assert_eq!(message.lines().count(), 2, "{message}");
    assert!(message.contains(&home.command().display().to_string()));
}

#[test]
fn uninstall_keeps_versions_and_aliases_where_nvmimic_made_the_folder() {
    let home = Home::new();
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let version = home.nvm_dir().join("versions/node/v20.1.0/bin");
    fs::create_dir_all(&version).unwrap();
    fs::write(version.join("node"), "node\n").unwrap();
    fs::create_dir_all(home.nvm_dir().join("alias")).unwrap();
    fs::write(home.nvm_dir().join("alias/default"), "20\n").unwrap();

    let out = home.nvmimic(&["self", "uninstall", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(version.join("node").is_file());
    assert!(home.nvm_dir().join("alias/default").is_file());
    for file in SCRIPTS {
        assert!(!home.nvm_dir().join(file).exists(), "{file}");
    }
}

#[test]
fn without_a_terminal_or_force_nothing_changes_and_the_plan_is_shown() {
    let home = Home::new();
    real_nvm(&home);
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let installed = fs::read(home.nvm_dir().join("nvm.sh")).unwrap();
    let dir = home.nvm_dir().display().to_string();

    for (args, plan) in [
        (
            &["self", "uninstall"][..],
            format!("Would remove nvmimic's scripts from {dir}."),
        ),
        (
            &["self", "eject"][..],
            format!("Would put nvm's own nvm.sh, nvm-exec, bash_completion back in {dir}."),
        ),
        (
            &["self", "uninstall", "--purge"][..],
            format!("Would delete {dir} and everything in it"),
        ),
    ] {
        let out = home.nvmimic(args);
        // The same answer every `self` command gives without a terminal.
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(text(&out.stdout).contains(&plan), "{}", text(&out.stdout));
        assert!(
            text(&out.stderr).contains("--force"),
            "{}",
            text(&out.stderr)
        );
        assert_eq!(fs::read(home.nvm_dir().join("nvm.sh")).unwrap(), installed);
        assert!(home.state().join("manifest").is_file());
    }
}

#[test]
fn eject_puts_nvm_back_and_keeps_the_program() {
    let home = Home::new();
    real_nvm(&home);
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));

    let out = home.nvmimic(&["self", "eject", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    for file in SCRIPTS {
        assert_eq!(
            fs::read(home.nvm_dir().join(file)).unwrap(),
            fs::read(reference().join(file)).unwrap(),
            "{file}"
        );
    }
    assert!(home.command().is_file());

    // A second eject has nothing left to do.
    let again = home.nvmimic(&["self", "eject", "--force"]);
    assert_eq!(again.status.code(), Some(1));
    assert!(text(&again.stderr).contains("already out of"));

    // `self install` takes over again, and uninstall still gives nvm back.
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("Taking over from the nvm in"));
    let nvm = home.shell("nvm --version");
    assert_eq!(text(&nvm.stdout), "0.40.8\n", "{}", text(&nvm.stderr));
    let out = home.nvmimic(&["self", "uninstall", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(
        fs::read(home.nvm_dir().join("nvm.sh")).unwrap(),
        fs::read(reference().join("nvm.sh")).unwrap()
    );
    assert!(!home.has_own_files());
}

#[test]
fn eject_with_no_nvm_to_go_back_to_changes_nothing() {
    let home = Home::new();
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let installed = fs::read(home.nvm_dir().join("nvm.sh")).unwrap();

    let out = home.nvmimic(&["self", "eject", "--force"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("there is no nvm to go back to"),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(fs::read(home.nvm_dir().join("nvm.sh")).unwrap(), installed);
}

#[test]
fn an_edited_script_is_renamed_and_nvms_goes_back() {
    let home = Home::new();
    real_nvm(&home);
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let script = home.nvm_dir().join("nvm.sh");
    let mut edited = fs::read_to_string(&script).unwrap();
    edited.push_str("# mine\n");
    fs::write(&script, &edited).unwrap();
    let saved = home.nvm_dir().join("nvm.sh.nvmimic-saved");
    let named = format!("{} to {}.", script.display(), saved.display());

    let out = home.nvmimic(&["self", "uninstall", "--dry-run"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).contains(&format!(
            "nvm.sh was edited after nvmimic installed it. Would rename {named}"
        )),
        "{}",
        text(&out.stdout)
    );
    assert!(!saved.exists());

    let out = home.nvmimic(&["self", "uninstall", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let lines: Vec<String> = text(&out.stdout)
        .lines()
        .filter(|line| line.contains("was edited"))
        .map(str::to_string)
        .collect();
    assert_eq!(
        lines,
        [format!(
            "nvm.sh was edited after nvmimic installed it. Renamed {named}"
        )]
    );
    assert_eq!(fs::read_to_string(&saved).unwrap(), edited);
    assert_eq!(
        fs::read(&script).unwrap(),
        fs::read(reference().join("nvm.sh")).unwrap()
    );
}

/// nvm's own installer, run again after nvmimic took over: it writes a newer
/// nvm's three scripts over nvmimic's.
fn newer_nvm_over_ours(home: &Home) {
    for file in SCRIPTS {
        let mut newer = fs::read_to_string(reference().join(file)).unwrap();
        if file != "nvm-exec" {
            newer.push_str("# a newer nvm\n");
        }
        fs::write(home.nvm_dir().join(file), newer).unwrap();
    }
}

#[test]
fn nvms_own_scripts_written_over_ours_are_left_alone() {
    for leaving in ["uninstall", "eject"] {
        let home = Home::new();
        real_nvm(&home);
        let out = home.nvmimic(&["self", "install", "--force"]);
        assert!(out.status.success(), "{}", text(&out.stderr));
        newer_nvm_over_ours(&home);
        let newer: Vec<Vec<u8>> = SCRIPTS
            .iter()
            .map(|file| fs::read(home.nvm_dir().join(file)).unwrap())
            .collect();

        let dry = home.nvmimic(&["self", leaving, "--dry-run"]);
        assert!(dry.status.success(), "{}", text(&dry.stderr));
        // nvm-exec is the same file in every nvm since 0.40, so the one in
        // the folder is the one in the backup, and there is nothing to say
        // about it.
        assert_eq!(
            text(&dry.stdout).matches("Would leave it alone.").count(),
            2,
            "{}",
            text(&dry.stdout)
        );

        let out = home.nvmimic(&["self", leaving, "--force"]);
        assert!(out.status.success(), "{}", text(&out.stderr));
        let said = text(&out.stdout);
        for (file, newer) in SCRIPTS.iter().zip(&newer) {
            let path = home.nvm_dir().join(file);
            assert_eq!(&fs::read(&path).unwrap(), newer, "{leaving} {file}");
            let changed = *file != "nvm-exec";
            assert_eq!(
                said.contains(&format!(
                    "{} isn't the file nvmimic put there. Left it alone.",
                    path.display()
                )),
                changed,
                "{said}"
            );
            assert!(
                !home
                    .nvm_dir()
                    .join(format!("{file}.nvmimic-saved"))
                    .exists()
            );
            // The older nvm nvmimic kept is still there, and stayed out. A
            // copy of what the folder holds anyway is not kept.
            assert_eq!(
                home.shared().join("backup").join(file).is_file(),
                changed,
                "{leaving} {file}"
            );
        }
        assert!(!said.contains("was edited"), "{said}");
    }
}

#[test]
fn a_script_with_no_fingerprint_is_compared_with_what_nvmimic_writes() {
    let home = Home::new();
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    // A manifest from before fingerprints, and one script copied in by hand.
    let manifest = home.state().join("manifest");
    let kept: String = fs::read_to_string(&manifest)
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with("fingerprint "))
        .map(|line| format!("{line}\n"))
        .collect();
    fs::write(&manifest, kept).unwrap();
    let completion = home.nvm_dir().join("bash_completion");
    let mut by_hand = fs::read_to_string(&completion).unwrap();
    by_hand.push_str("# by hand\n");
    fs::write(&completion, &by_hand).unwrap();

    let out = home.nvmimic(&["self", "uninstall", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(
        fs::read_to_string(home.nvm_dir().join("bash_completion.nvmimic-saved")).unwrap(),
        by_hand
    );
    assert!(!home.nvm_dir().join("nvm.sh").exists());
    assert!(!home.nvm_dir().join("nvm.sh.nvmimic-saved").exists());
}

#[test]
fn help_reads_right_with_and_without_an_nvm() {
    let home = Home::new();
    for args in [&[][..], &["help"], &["--help"]] {
        let out = home.nvmimic(args);
        assert!(out.status.success());
        assert!(text(&out.stdout).contains("self install"));
    }
    let out = home.nvmimic(&["self", "install", "--help"]);
    let help = text(&out.stdout);
    assert!(out.status.success());
    assert!(help.starts_with("usage: nvmimic self install"), "{help}");
    assert!(!home.has_own_files(), "asking for help installs nothing");
    // The internal entry point stays out of every help text.
    for args in [&["help"][..], &["self", "--help"]] {
        assert!(!text(&home.nvmimic(args).stdout).contains("__internal"));
    }
}

#[test]
fn every_help_screen_of_ours_is_short() {
    let home = Home::new();
    let mut screens: Vec<Vec<&str>> = vec![vec!["--help"], vec!["self"], vec!["init", "--help"]];
    for command in SELF_COMMANDS {
        screens.push(vec!["self", command, "--help"]);
    }
    for args in &screens {
        let out = home.nvmimic(args);
        let help = text(&out.stdout);
        assert!(out.status.success(), "{args:?}");
        assert!(help.starts_with("usage: nvmimic"), "{args:?}: {help}");
        assert!(help.lines().count() <= 14, "{args:?} is long:\n{help}");
    }
}

#[test]
fn self_alone_lists_its_commands_in_order() {
    let home = Home::new();
    let index = text(&home.nvmimic(&["self", "--help"]).stdout);
    assert_eq!(index, text(&home.nvmimic(&["self"]).stdout));
    let commands: Vec<&str> = index
        .lines()
        .filter_map(|line| line.strip_prefix("  ")?.split_whitespace().next())
        .filter(|word| !word.starts_with('-'))
        .collect();
    assert_eq!(commands, SELF_COMMANDS);
    for flag in ["--force", "--dry-run"] {
        assert!(index.contains(flag), "{index}");
    }
    assert!(index.ends_with("Run nvmimic self <command> --help for details.\n"));
}

#[test]
fn a_mistake_gets_one_line_that_says_where_to_look() {
    let home = Home::new();
    let mistakes: [(&[&str], &str); 5] = [
        (
            &["self", "install", "--bogus"],
            "nvmimic: self install has no option --bogus. Run nvmimic self install --help.\n",
        ),
        (
            &["self", "eject", "--purge"],
            "nvmimic: self eject has no option --purge. Run nvmimic self eject --help.\n",
        ),
        (
            &["frobnicate"],
            "nvmimic: there is no command frobnicate. Run nvmimic --help.\n",
        ),
        (
            &["self", "frobnicate"],
            "nvmimic: there is no command self frobnicate. Run nvmimic self --help.\n",
        ),
        (
            &["init", "fish"],
            "nvmimic: init doesn't know the shell fish. It knows bash, zsh, sh, dash. Run nvmimic init --help.\n",
        ),
    ];
    for (args, message) in mistakes {
        let out = home.nvmimic(args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert_eq!(text(&out.stderr), message);
        assert_eq!(text(&out.stdout), "");
    }
}

const SELF_COMMANDS: [&str; 5] = ["install", "eject", "uninstall", "update", "rollback"];

#[test]
fn messages_say_which_situation_this_is() {
    let home = Home::new();
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(text(&out.stdout).contains("sets one up from scratch"));
    let out = home.nvmimic(&["self", "uninstall", "--force"]);
    let said = text(&out.stdout);
    assert!(said.contains("there is no `nvm` command"), "{said}");
    assert!(!said.contains("nvm is back"), "{said}");

    let home = Home::new();
    real_nvm(&home);
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(!text(&out.stdout).contains("from scratch"));
    let out = home.nvmimic(&["self", "uninstall", "--force"]);
    assert!(text(&out.stdout).contains("nvm is back"));
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
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    // A script is never remembered, so a compiled program stands in for
    // node: nvmimic itself, which answers `--version` with one line. One
    // `nvm use` makes nvmimic remember that answer.
    fs::remove_file(&node).unwrap();
    fs::copy(env!("CARGO_BIN_EXE_nvmimic"), &node).unwrap();
    let _ = home.shell("nvm use 22");
    let after: Vec<PathBuf> = snapshot(&home.nvm_dir())
        .into_iter()
        .map(|(path, _)| path)
        .collect();
    assert_eq!(after, before);
    for dir in home.own() {
        assert!(dir.is_dir(), "{} is missing", dir.display());
    }
    let [_, _, cache, _] = home.own();
    assert!(
        fs::read_dir(cache.join("node-version")).unwrap().count() == 1,
        "node's answer was remembered outside $NVM_DIR"
    );
}

#[test]
fn a_script_named_node_is_not_remembered() {
    let home = Home::new();
    let node = home.nvm_dir().join("versions/node/v22.1.0/bin/node");
    fs::create_dir_all(node.parent().unwrap()).unwrap();
    fs::write(&node, "#!/bin/sh\necho v22.1.0\n").unwrap();
    fs::set_permissions(&node, fs::Permissions::from_mode(0o755)).unwrap();
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let out = home.shell("nvm use 22 >/dev/null; nvm current");
    assert_eq!(text(&out.stdout).trim(), "v22.1.0");
    let [_, _, cache, _] = home.own();
    assert!(
        !cache.join("node-version").exists(),
        "a script's answer was remembered"
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
        // The record shared with sister tools has a variable of its own,
        // with no program's name in it.
        ("NVMIMIC_SHARED_DIR", elsewhere.join("shared")),
    ];
    let vars: Vec<(&str, &Path)> = vars.iter().map(|(k, v)| (*k, v.as_path())).collect();
    // Something of the user's own in the data folder has to survive.
    fs::create_dir_all(elsewhere.join("data")).unwrap();
    fs::write(elsewhere.join("data/mine"), "mine\n").unwrap();

    let out = home.nvmimic_with(&["self", "install", "--force"], &vars);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(elsewhere.join("tools/nvmimic").is_file());
    assert!(elsewhere.join("state/manifest").is_file());
    assert!(elsewhere.join("shared/record").is_file());
    assert!(!home.has_own_files());
    let nvm = home.shell("nvm --version");
    assert_eq!(text(&nvm.stdout), "0.40.8\n", "{}", text(&nvm.stderr));

    let out = home.nvmimic_with(&["self", "uninstall", "--force"], &vars);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!elsewhere.join("tools/nvmimic").exists());
    assert!(!elsewhere.join("state").exists());
    assert!(!elsewhere.join("shared").exists());
    assert_eq!(
        fs::read_to_string(elsewhere.join("data/mine")).unwrap(),
        "mine\n"
    );
}

#[test]
fn takeover_keeps_versions_and_uninstall_restores_nvm() {
    let home = Home::new();
    real_nvm(&home);
    let out = home.nvmimic(&["self", "install", "--force"]);
    let stdout = text(&out.stdout);
    assert!(out.status.success(), "{stdout}{}", text(&out.stderr));
    assert!(stdout.contains("drop-in replacement"));
    // `--force` takes the default of the question about a full copy.
    assert!(
        stdout.contains("to a backup folder first? [y/N] n (--force)"),
        "{stdout}"
    );
    assert!(!home.root.join(".nvm.backup-1").exists());

    let dir = home.nvm_dir();
    assert!(dir.join("versions/node/v22.1.0/bin").is_dir());
    assert_eq!(
        fs::read_to_string(dir.join("alias/default")).unwrap(),
        "22\n"
    );
    assert_eq!(
        fs::read(home.shared().join("backup/nvm.sh")).unwrap(),
        fs::read(reference().join("nvm.sh")).unwrap()
    );
    let nvm = home.shell("nvm alias default");
    assert_eq!(
        text(&nvm.stdout),
        "default -> 22 (-> v22.1.0 *)\n",
        "{}",
        text(&nvm.stderr)
    );

    let out = home.nvmimic(&["self", "uninstall", "--force"]);
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
    let out = home.nvmimic(&["self", "install", "--force", "--dry-run"]);
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
fn a_dry_run_lists_its_steps_and_never_asks() {
    let home = Home::new();
    real_nvm(&home);
    let out = home.nvmimic(&["self", "install", "--dry-run"]);
    let said = text(&out.stdout);
    assert!(out.status.success(), "{said}{}", text(&out.stderr));
    assert!(said.contains("Would install nvmimic"), "{said}");
    assert!(said.contains("Would keep a copy of"), "{said}");
    assert!(!said.contains("[y/N]"), "{said}");
    assert!(!home.has_own_files());
    assert!(!home.root.join(".nvm.backup-1").exists());
}

#[test]
fn git_installs_get_a_warning() {
    let home = Home::new();
    real_nvm(&home);
    fs::create_dir_all(home.nvm_dir().join(".git")).unwrap();
    let out = home.nvmimic(&["self", "install", "--force"]);
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
    assert!(text(&out.stderr).contains("--force"));
}

/// A Homebrew under `<home>/brew` with a stand-in `brew` that does what
/// the real one does to `opt/nvm` for the three formulas `self install`
/// names, and writes down every call in `brew.log` beside it. Homebrew's own
/// nvm is installed, and `$NVM_DIR` has the links its `nvm.sh` makes. A
/// call listed in `brew.fail` exits 1 and changes nothing.
///
/// The real `brew` is never run by a test (decisions/0021).
struct Brew {
    prefix: PathBuf,
}

impl Brew {
    fn new(home: &Home) -> Brew {
        let prefix = home.root.join("brew");
        let stand_in = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shell/homebrew");
        fs::create_dir_all(prefix.join("bin")).unwrap();
        let script = format!(
            r#"#!/bin/sh
prefix='{prefix}'
echo "$*" >> "$prefix/brew.log"
grep -qx "$*" "$prefix/brew.fail" 2>/dev/null && exit 1
nvm="$prefix/opt/nvm"
case "$*" in
  'uninstall nvm')
    [ -d "$nvm" ] || exit 1
    rm -rf "$nvm"
    ;;
  'install homebrew/core/nvm'|'install nvm')
    [ -d "$nvm" ] && exit 1
    mkdir -p "$nvm/libexec" "$nvm/etc/bash_completion.d"
    cp '{reference}/nvm.sh' '{reference}/nvm-exec' "$nvm/libexec/"
    cp '{reference}/bash_completion' "$nvm/etc/bash_completion.d/nvm"
    printf '%s\n' '[ -z "$NVM_DIR" ] && export NVM_DIR="$HOME/.nvm"' \
      "\\. $nvm/libexec/nvm.sh" \
      '[ -e "$NVM_DIR/nvm.sh" ] || ln -s '"$nvm"'/libexec/nvm.sh "$NVM_DIR/nvm.sh"' \
      '[ -e "$NVM_DIR/nvm-exec" ] || ln -s '"$nvm"'/libexec/nvm-exec "$NVM_DIR/nvm-exec"' \
      > "$nvm/nvm.sh"
    ;;
  'install tsilenzio/tap/nvm')
    [ -d "$nvm" ] && exit 1
    mkdir -p "$nvm/etc/bash_completion.d"
    cp '{stand_in}/nvm.sh' "$nvm/nvm.sh"
    cp '{stand_in}/bash_completion' "$nvm/etc/bash_completion.d/nvm"
    ;;
  *) exit 1 ;;
esac
"#,
            prefix = prefix.display(),
            reference = reference().display(),
            stand_in = stand_in.display(),
        );
        // Written beside it and placed with `cp`: Linux refuses to start a
        // program that any process still has open for writing, and another
        // test forking at the wrong moment would hold a file written here.
        let written = prefix.join("brew.script");
        fs::write(&written, script).unwrap();
        fs::set_permissions(&written, fs::Permissions::from_mode(0o755)).unwrap();
        let placed = Command::new("cp")
            .arg(&written)
            .arg(prefix.join("bin/brew"))
            .status()
            .unwrap();
        assert!(placed.success());
        let brew = Brew { prefix };
        brew.run("install homebrew/core/nvm");
        // What sourcing Homebrew's nvm.sh leaves in `$NVM_DIR`.
        fs::create_dir_all(home.nvm_dir().join("alias")).unwrap();
        fs::write(home.nvm_dir().join("alias/default"), "22\n").unwrap();
        for file in ["nvm.sh", "nvm-exec"] {
            std::os::unix::fs::symlink(
                brew.prefix.join("opt/nvm/libexec").join(file),
                home.nvm_dir().join(file),
            )
            .unwrap();
        }
        let _ = fs::remove_file(brew.log_path());
        brew
    }

    fn run(&self, call: &str) {
        let out = Command::new(self.prefix.join("bin/brew"))
            .args(call.split(' '))
            .output()
            .unwrap();
        assert!(out.status.success(), "brew {call}");
    }

    fn log_path(&self) -> PathBuf {
        self.prefix.join("brew.log")
    }

    /// The calls since the last look, and the log emptied.
    fn calls(&self) -> Vec<String> {
        let log = fs::read_to_string(self.log_path()).unwrap_or_default();
        let _ = fs::remove_file(self.log_path());
        log.lines().map(str::to_string).collect()
    }

    fn fail(&self, call: &str) {
        fs::write(self.prefix.join("brew.fail"), format!("{call}\n")).unwrap();
    }

    fn holds_stand_in(&self) -> bool {
        fs::read_to_string(self.prefix.join("opt/nvm/nvm.sh"))
            .is_ok_and(|text| text.contains("nvm stand-in from tsilenzio/tap"))
    }

    fn holds_official(&self) -> bool {
        self.prefix.join("opt/nvm/libexec/nvm.sh").is_file()
    }

    fn vars(&self) -> [(&str, &Path); 1] {
        [("HOMEBREW_PREFIX", self.prefix.as_path())]
    }

    /// Which nvm a shell gets from the line in a startup file that loads
    /// Homebrew's nvm: `ours`, `official` or `none`, after nvm's version.
    fn loads(&self, home: &Home, shell: &str) -> String {
        let nvm_sh = self.prefix.join("opt/nvm/nvm.sh");
        let out = Command::new(shell)
            .arg("-c")
            .arg(format!(
                "[ -s \"{0}\" ] && \\. \"{0}\"\n\
                 command -v nvm >/dev/null || {{ echo none; exit; }}\n\
                 nvm --version\n\
                 command -v __nvmimic_handoff >/dev/null && echo ours || echo official",
                nvm_sh.display()
            ))
            .env_clear()
            .env("HOME", &home.root)
            .env("NVM_DIR", home.nvm_dir())
            .env("PATH", "/usr/bin:/bin")
            .output()
            .unwrap();
        text(&out.stdout).trim().replace('\n', " ")
    }
}

/// The line a startup file holds when a tool other than the user manages
/// it, as on the owner's work laptop.
const LOADS_HOMEBREWS_NVM: &str =
    "[ -s \"/opt/homebrew/opt/nvm/nvm.sh\" ] && \\. \"/opt/homebrew/opt/nvm/nvm.sh\"\n";

#[test]
fn a_startup_line_that_loads_homebrews_nvm_gets_the_stand_in_and_leaving_puts_homebrews_back() {
    for leaving in ["eject", "uninstall"] {
        let home = Home::new();
        let brew = Brew::new(&home);
        let line = format!("# nvm\n{}", LOADS_HOMEBREWS_NVM)
            .replace("/opt/homebrew", &brew.prefix.display().to_string());
        fs::write(home.root.join(".zshrc"), line).unwrap();
        let nvm_before = snapshot(&home.nvm_dir());
        let brew_before = snapshot(&brew.prefix.join("opt"));

        let out = home.nvmimic_with(&["self", "install", "--force"], &brew.vars());
        let said = text(&out.stdout);
        assert!(out.status.success(), "{said}{}", text(&out.stderr));
        assert!(
            said.contains(&format!(
                "  {}, line 2:\n",
                home.root.join(".zshrc").display()
            )),
            "{said}"
        );
        assert!(said.contains("Run `brew uninstall nvm`, then `brew install tsilenzio/tap/nvm` now? [y/N] y (--force)"), "{said}");
        assert_eq!(brew.calls(), ["uninstall nvm", "install tsilenzio/tap/nvm"]);
        assert!(brew.holds_stand_in());
        for shell in ["/bin/zsh", "/bin/bash"] {
            assert_eq!(brew.loads(&home, shell), "0.40.8 ours", "{shell}");
        }

        let out = home.nvmimic_with(&["self", leaving, "--force"], &brew.vars());
        let said = text(&out.stdout);
        assert!(
            out.status.success(),
            "{leaving}: {said}{}",
            text(&out.stderr)
        );
        assert!(
            said.contains("Homebrew's own nvm is installed again."),
            "{said}"
        );
        assert_eq!(
            brew.calls(),
            ["uninstall nvm", "install homebrew/core/nvm"],
            "{leaving}"
        );
        assert_eq!(snapshot(&home.nvm_dir()), nvm_before, "{leaving}");
        assert_eq!(snapshot(&brew.prefix.join("opt")), brew_before, "{leaving}");
        for shell in ["/bin/zsh", "/bin/bash"] {
            assert_eq!(
                brew.loads(&home, shell),
                "0.40.8 official",
                "{leaving} {shell}"
            );
        }

        // After an eject, the uninstall has nothing of Homebrew's to change.
        if leaving == "eject" {
            let out = home.nvmimic_with(&["self", "uninstall", "--force"], &brew.vars());
            assert!(out.status.success(), "{}", text(&out.stderr));
            assert!(brew.calls().is_empty());
            assert_eq!(snapshot(&home.nvm_dir()), nvm_before);
        }
        assert!(!home.has_own_files() || leaving == "eject");
    }
}

#[test]
fn a_line_in_a_file_that_a_startup_file_sources_counts() {
    let home = Home::new();
    let brew = Brew::new(&home);
    fs::write(
        home.root.join(".zshrc"),
        "export EDITOR=vi\n[ -f ~/.zshrc_platform ] && source ~/.zshrc_platform\n",
    )
    .unwrap();
    fs::write(
        home.root.join(".zshrc_platform"),
        format!("# BEGIN managed block\n{LOADS_HOMEBREWS_NVM}# END managed block\n"),
    )
    .unwrap();

    let out = home.nvmimic_with(&["self", "install", "--force"], &brew.vars());
    let said = text(&out.stdout);
    assert!(out.status.success(), "{said}{}", text(&out.stderr));
    assert!(
        said.contains(&format!(
            "This line in your shell's startup files loads it:\n  {}, line 2, which {} loads:\n    [ -s \"/opt/homebrew/opt/nvm/nvm.sh\"",
            home.root.join(".zshrc_platform").display(),
            home.root.join(".zshrc").display()
        )),
        "{said}"
    );
    assert!(
        said.contains("Open a new shell to start using it."),
        "{said}"
    );
    assert_eq!(brew.calls(), ["uninstall nvm", "install tsilenzio/tap/nvm"]);
}

/// zsh reads its startup files from `ZDOTDIR` when that is set, so a line
/// there counts, and only then.
#[test]
fn a_line_in_zdotdir_counts_when_zdotdir_is_set() {
    for set in [false, true] {
        let home = Home::new();
        let brew = Brew::new(&home);
        let zdotdir = home.root.join(".config/zsh");
        fs::create_dir_all(&zdotdir).unwrap();
        fs::write(zdotdir.join(".zshrc"), LOADS_HOMEBREWS_NVM).unwrap();
        let mut vars = brew.vars().to_vec();
        if set {
            vars.push(("ZDOTDIR", zdotdir.as_path()));
        }
        let out = home.nvmimic_with(&["self", "install", "--force"], &vars);
        let said = text(&out.stdout);
        assert!(out.status.success(), "{said}{}", text(&out.stderr));
        if set {
            assert!(
                said.contains(&format!(
                    "  {}, line 1:\n",
                    zdotdir.join(".zshrc").display()
                )),
                "{said}"
            );
            assert_eq!(brew.calls(), ["uninstall nvm", "install tsilenzio/tap/nvm"]);
        } else {
            assert!(!said.contains("startup files load"), "{said}");
            assert_eq!(brew.calls(), ["uninstall nvm"]);
        }
    }
}

/// The sequence from the owner's work laptop: Homebrew's nvm with no line
/// of its own in a startup file, taken over, then ejected. Before 1.0.1 the
/// eject put copies back and left Homebrew's nvm uninstalled.
#[test]
fn with_no_startup_line_homebrews_nvm_is_only_removed_and_eject_brings_it_back_with_its_links() {
    let home = Home::new();
    let brew = Brew::new(&home);
    let nvm_before = snapshot(&home.nvm_dir());
    let brew_before = snapshot(&brew.prefix.join("opt"));

    let out = home.nvmimic_with(&["self", "install", "--force"], &brew.vars());
    let said = text(&out.stdout);
    assert!(out.status.success(), "{said}{}", text(&out.stderr));
    assert!(
        said.contains("Run `brew uninstall nvm` now? [y/N] y (--force)"),
        "{said}"
    );
    assert_eq!(brew.calls(), ["uninstall nvm"]);
    assert!(!brew.prefix.join("opt/nvm").exists());

    let out = home.nvmimic_with(&["self", "eject", "--force"], &brew.vars());
    let said = text(&out.stdout);
    assert!(out.status.success(), "{said}{}", text(&out.stderr));
    assert!(
        said.contains("Homebrew's own nvm is installed again."),
        "{said}"
    );
    assert_eq!(brew.calls(), ["install homebrew/core/nvm"]);
    assert_eq!(snapshot(&home.nvm_dir()), nvm_before);
    assert_eq!(snapshot(&brew.prefix.join("opt")), brew_before);
    assert!(is_link(&home.nvm_dir().join("nvm.sh")));
    assert!(is_link(&home.nvm_dir().join("nvm-exec")));
}

/// An install made by 1.0.0 has no line in the shared record for
/// Homebrew. A link it kept that points into Homebrew's nvm folder says
/// that Homebrew's nvm was removed.
#[test]
fn an_install_made_by_1_0_0_gets_homebrews_nvm_back_by_its_links() {
    for leaving in ["eject", "uninstall"] {
        let home = Home::new();
        let brew = Brew::new(&home);
        let nvm_before = snapshot(&home.nvm_dir());
        let out = home.nvmimic_with(&["self", "install", "--force"], &brew.vars());
        assert!(out.status.success(), "{}", text(&out.stderr));
        brew.calls();
        let record = home.shared().join("record");
        let written = fs::read_to_string(&record).unwrap();
        assert!(written.contains("homebrew-removed "), "{written}");
        let old: String = written
            .lines()
            .filter(|line| !line.starts_with("homebrew-"))
            .map(|line| format!("{line}\n"))
            .collect();
        assert!(
            old.contains(&format!(
                "link nvm.sh {}/opt/nvm/libexec/nvm.sh",
                brew.prefix.display()
            )),
            "{old}"
        );
        fs::write(&record, old).unwrap();

        let out = home.nvmimic_with(&["self", leaving, "--force"], &brew.vars());
        assert!(out.status.success(), "{leaving}: {}", text(&out.stderr));
        assert_eq!(brew.calls(), ["install homebrew/core/nvm"], "{leaving}");
        assert_eq!(snapshot(&home.nvm_dir()), nvm_before, "{leaving}");
    }
}

#[test]
fn when_the_stand_in_cannot_be_installed_homebrews_nvm_comes_back_and_nothing_changes() {
    let home = Home::new();
    let brew = Brew::new(&home);
    fs::write(home.root.join(".bashrc"), LOADS_HOMEBREWS_NVM).unwrap();
    brew.fail("install tsilenzio/tap/nvm");
    let nvm_before = snapshot(&home.nvm_dir());
    let brew_before = snapshot(&brew.prefix.join("opt"));

    let out = home.nvmimic_with(&["self", "install", "--force"], &brew.vars());
    assert_eq!(out.status.code(), Some(1));
    let said = text(&out.stderr);
    assert!(
        said.contains("`brew install tsilenzio/tap/nvm` did not finish, so Homebrew's own nvm was put back. Nothing else was changed."),
        "{said}"
    );
    assert_eq!(
        brew.calls(),
        [
            "uninstall nvm",
            "install tsilenzio/tap/nvm",
            "install homebrew/core/nvm"
        ]
    );
    assert_eq!(snapshot(&home.nvm_dir()), nvm_before);
    assert_eq!(snapshot(&brew.prefix.join("opt")), brew_before);
    assert!(!home.has_own_files());
    assert_eq!(brew.loads(&home, "/bin/bash"), "0.40.8 official");
}

#[test]
fn when_homebrews_nvm_cannot_come_back_its_scripts_come_back_as_copies() {
    let home = Home::new();
    let brew = Brew::new(&home);
    fs::write(home.root.join(".zshrc"), LOADS_HOMEBREWS_NVM).unwrap();
    let out = home.nvmimic_with(&["self", "install", "--force"], &brew.vars());
    assert!(out.status.success(), "{}", text(&out.stderr));
    brew.calls();
    brew.fail("install homebrew/core/nvm");

    let out = home.nvmimic_with(&["self", "eject", "--force"], &brew.vars());
    let said = text(&out.stdout);
    assert!(out.status.success(), "{said}{}", text(&out.stderr));
    assert_eq!(
        brew.calls(),
        [
            "uninstall nvm",
            "install homebrew/core/nvm",
            "install tsilenzio/tap/nvm"
        ]
    );
    assert!(
        said.contains("`brew install homebrew/core/nvm` did not finish. Official nvm's scripts go back as copies. To finish the job, run:\n  brew uninstall nvm\n  brew install nvm\n"),
        "{said}"
    );
    for file in ["nvm.sh", "nvm-exec"] {
        let path = home.nvm_dir().join(file);
        assert!(!is_link(&path), "{file}");
        assert_eq!(
            fs::read(&path).unwrap(),
            fs::read(reference().join(file)).unwrap()
        );
    }
    // The stand-in went back in, and loads official nvm from `$NVM_DIR`.
    assert!(brew.holds_stand_in());
    assert_eq!(brew.loads(&home, "/bin/zsh"), "0.40.8 official");
}

#[test]
fn a_dry_run_lists_the_brew_commands_and_runs_none() {
    let home = Home::new();
    let brew = Brew::new(&home);
    fs::write(home.root.join(".zshrc"), LOADS_HOMEBREWS_NVM).unwrap();
    let out = home.nvmimic_with(&["self", "install", "--dry-run"], &brew.vars());
    let said = text(&out.stdout);
    assert!(out.status.success(), "{said}{}", text(&out.stderr));
    assert!(
        said.contains("Would run `brew uninstall nvm`, then `brew install tsilenzio/tap/nvm`.\n"),
        "{said}"
    );
    assert!(brew.calls().is_empty());

    let out = home.nvmimic_with(&["self", "install", "--force"], &brew.vars());
    assert!(out.status.success(), "{}", text(&out.stderr));
    brew.calls();
    for leaving in ["eject", "uninstall"] {
        let out = home.nvmimic_with(&["self", leaving, "--dry-run"], &brew.vars());
        let said = text(&out.stdout);
        assert!(out.status.success(), "{said}{}", text(&out.stderr));
        assert!(
            said.contains("Would run `brew uninstall nvm`, then `brew install homebrew/core/nvm`, to put Homebrew's own nvm back in place of the stand-in.\n"),
            "{leaving}: {said}"
        );
        assert!(brew.calls().is_empty());
    }
    // Without a terminal and without --force, the steps are shown and
    // nothing runs.
    let out = home.nvmimic_with(&["self", "uninstall"], &brew.vars());
    assert!(
        text(&out.stdout)
            .contains("Would run `brew uninstall nvm`, then `brew install homebrew/core/nvm`")
    );
    assert!(brew.calls().is_empty());
}

#[test]
fn a_stand_in_that_is_already_there_is_left_alone() {
    let home = Home::new();
    let brew = Brew::new(&home);
    fs::write(home.root.join(".zshrc"), LOADS_HOMEBREWS_NVM).unwrap();
    brew.run("uninstall nvm");
    brew.run("install tsilenzio/tap/nvm");
    brew.calls();

    let out = home.nvmimic_with(&["self", "install", "--force"], &brew.vars());
    let said = text(&out.stdout);
    assert!(out.status.success(), "{said}{}", text(&out.stderr));
    assert!(!said.contains("Homebrew"), "{said}");
    let out = home.nvmimic_with(&["self", "uninstall", "--force"], &brew.vars());
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(brew.calls().is_empty());
    assert!(brew.holds_stand_in());
}

/// An install stopped after `brew uninstall nvm` and before the stand-in
/// went in. The next install puts the stand-in in.
#[test]
fn a_swap_that_was_stopped_halfway_is_finished_by_the_next_install() {
    let home = Home::new();
    let brew = Brew::new(&home);
    fs::write(home.root.join(".zshrc"), LOADS_HOMEBREWS_NVM).unwrap();
    let out = home.nvmimic_with(&["self", "install", "--force"], &brew.vars());
    assert!(out.status.success(), "{}", text(&out.stderr));
    brew.run("uninstall nvm");
    brew.calls();

    let out = home.nvmimic_with(&["self", "install", "--force"], &brew.vars());
    let said = text(&out.stdout);
    assert!(out.status.success(), "{said}{}", text(&out.stderr));
    assert!(said.contains("It goes in now."), "{said}");
    assert_eq!(brew.calls(), ["install tsilenzio/tap/nvm"]);
    assert!(brew.holds_stand_in());
    assert!(!brew.holds_official());
}

/// A `brew` that only writes down what it was asked, and the log it writes.
fn stand_in_brew(dir: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    fs::create_dir_all(dir).unwrap();
    let log = dir.join("brew.log");
    let brew = dir.join("brew");
    fs::write(
        &brew,
        format!("#!/bin/sh\necho \"$@\" >> '{}'\n", log.display()),
    )
    .unwrap();
    fs::set_permissions(&brew, fs::Permissions::from_mode(0o755)).unwrap();
    log
}

/// A stand-in `brew` is first on `PATH` in every run here, where a real one
/// would be for someone with Homebrew. None of the runs may start it.
#[test]
fn no_brew_is_started_that_the_environment_did_not_name() {
    let run = |home: &Home, prefix: Option<&Path>| {
        let on_path = home.root.join("on-path");
        let log = stand_in_brew(&on_path);
        let mut command = Command::new(env!("CARGO_BIN_EXE_nvmimic"));
        command
            .args(["self", "install", "--force"])
            .env_clear()
            .env("HOME", &home.root)
            .env("PATH", format!("{}:/usr/bin:/bin", on_path.display()))
            .stdin(std::process::Stdio::null());
        if let Some(prefix) = prefix {
            command.env("HOMEBREW_PREFIX", prefix);
        }
        (command.output().unwrap(), log)
    };

    // A throwaway home and no `HOMEBREW_PREFIX`: Homebrew's usual places
    // are not looked in, so whatever this machine has there is not found.
    let home = Home::new();
    real_nvm(&home);
    let (out, log) = run(&home, None);
    let stdout = text(&out.stdout);
    assert!(out.status.success(), "{stdout}{}", text(&out.stderr));
    assert!(!stdout.contains("Homebrew"), "{stdout}");
    assert!(!log.exists());

    // A prefix that holds nvm and no `brew`: the one on `PATH` is not a
    // substitute. The install stops with nothing changed.
    let home = Home::new();
    real_nvm(&home);
    let prefix = home.root.join("brew");
    fs::create_dir_all(prefix.join("opt/nvm")).unwrap();
    let before = snapshot(&home.nvm_dir());
    let (out, log) = run(&home, Some(&prefix));
    assert_eq!(out.status.code(), Some(1));
    let said = text(&out.stderr);
    assert!(said.contains("there is no brew at"), "{said}");
    assert!(said.contains("Nothing was changed."), "{said}");
    assert!(!log.exists());
    assert_eq!(snapshot(&home.nvm_dir()), before);
    assert!(!home.root.join(".local").exists());

    // With a `brew` under the prefix, that one is run and no other.
    let named = stand_in_brew(&prefix.join("bin"));
    let (out, log) = run(&home, Some(&prefix));
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(fs::read_to_string(named).unwrap(), "uninstall nvm\n");
    assert!(!log.exists());
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

    let out = home.nvmimic(&["self", "install", "--force"]);
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

    let out = home.nvmimic(&["self", "uninstall", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(status(), before, "same as before the takeover");
}

#[test]
fn install_puts_the_program_in_local_bin() {
    let home = Home::new();
    let command = home.command();
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(fs::symlink_metadata(&command).unwrap().is_file());
    assert!(text(&out.stdout).contains("isn't on your PATH yet"));
    // Nothing named nvm goes on PATH: official nvm is only a shell function.
    assert!(!home.root.join(".local/bin/nvm").exists());

    // The installed program can install again, and stays the same file.
    let again = Command::new(&command)
        .args(["self", "install", "--force"])
        .env_clear()
        .env("HOME", &home.root)
        .env("PATH", "/usr/bin:/bin")
        .env("HOMEBREW_PREFIX", home.root.join("no-homebrew"))
        .output()
        .unwrap();
    assert!(again.status.success(), "{}", text(&again.stderr));
    assert!(command.is_file());

    let out = home.nvmimic(&["self", "uninstall", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(fs::symlink_metadata(&command).is_err());
}

#[test]
fn a_nvmimic_command_that_is_someone_elses_is_left_alone() {
    let home = Home::new();
    let command = home.command();
    fs::create_dir_all(command.parent().unwrap()).unwrap();
    fs::write(&command, "mine\n").unwrap();

    let out = home.nvmimic(&["self", "install", "--force"]);
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
    let out = home.nvmimic_with(&["self", "install", "--force"], &vars);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(elsewhere.is_file());
    let out = home.nvmimic_with(&["self", "uninstall", "--force"], &vars);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!elsewhere.exists());
    assert_eq!(fs::read_to_string(&command).unwrap(), "mine\n");
}

#[test]
fn a_second_nvm_dir_needs_an_uninstall_first() {
    let home = Home::new();
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let other = home.root.join("other-nvm");
    let out = home.nvmimic_with(
        &["self", "install", "--force"],
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

    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let record = fs::read_to_string(home.shared().join("record")).unwrap();
    for file in SCRIPTS {
        assert!(record.contains(&format!("backup {file}\n")), "{record}");
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
        .args(["self", "install", "--force"])
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
            fs::read(home.shared().join("backup").join(file)).unwrap(),
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

    let out = home.nvmimic(&["self", "uninstall", "--force"]);
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
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    nvm_installer(&home, "script");
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let out = home.nvmimic(&["self", "uninstall", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    for file in SCRIPTS {
        assert_eq!(
            fs::read(dir.join(file)).unwrap(),
            fs::read(reference().join(file)).unwrap(),
            "{file} is nvm's and stays"
        );
    }
}

#[cfg(target_os = "macos")]
/// A folder laid out like a release's downloads, holding a stand-in for a
/// newer nvmimic: a script that reports `version` and whose `self install`
/// exits with `install_status`.
fn fake_release(home: &Home, version: &str, install_status: u8) -> PathBuf {
    let release = home
        .root
        .join(format!("release-{version}-{install_status}"));
    let staging = release.join("nvmimic");
    fs::create_dir_all(&staging).unwrap();
    let program = staging.join("nvmimic");
    fs::write(
        &program,
        format!(
            "#!/bin/sh\ncase \"$1\" in\n  --version) echo {version} ;;\n  self) exit {install_status} ;;\nesac\n"
        ),
    )
    .unwrap();
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
    let archive = "nvmimic-darwin-universal.tar.gz";
    let packed = Command::new("tar")
        .arg("-czf")
        .arg(release.join(archive))
        .arg("-C")
        .arg(&release)
        .arg("nvmimic")
        .status()
        .unwrap();
    assert!(packed.success());
    let sum = Command::new("shasum")
        .args(["-a", "256"])
        .arg(release.join(archive))
        .output()
        .unwrap();
    let sum = text(&sum.stdout);
    let sum = sum.split_whitespace().next().unwrap();
    fs::write(release.join("SHA256SUMS"), format!("{sum}  {archive}\n")).unwrap();
    fs::write(release.join("VERSION"), format!("{version}\n")).unwrap();
    release
}

#[cfg(target_os = "macos")]
/// Runs a `self` command against a stand-in release folder.
fn with_release(home: &Home, release: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nvmimic"))
        .args(args)
        .env_clear()
        .env("HOME", &home.root)
        .env("PATH", "/usr/bin:/bin")
        .env("HOMEBREW_PREFIX", home.root.join("no-homebrew"))
        .env(
            "NVMIMIC_RELEASES_URL",
            format!("file://{}", release.display()),
        )
        .env("NVMIMIC_UPDATE_UNSIGNED", "1")
        .output()
        .unwrap()
}

#[cfg(target_os = "macos")]
#[test]
fn update_swaps_the_program_and_rollback_brings_it_back() {
    let home = Home::new();
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let original = fs::read(home.command()).unwrap();
    let ours = env!("CARGO_PKG_VERSION");
    let kept = home.state().join(format!("previous/nvmimic-{ours}"));

    let release = fake_release(&home, "9.9.9", 0);
    let out = with_release(&home, &release, &["self", "update", "--force", "--dry-run"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(
        fs::read(home.command()).unwrap(),
        original,
        "a dry run changes nothing"
    );

    let out = with_release(&home, &release, &["self", "update", "--force"]);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    let now = Command::new(home.command())
        .arg("--version")
        .output()
        .unwrap();
    assert_eq!(text(&now.stdout), "9.9.9\n");
    assert_eq!(
        fs::read(&kept).unwrap(),
        original,
        "the earlier program is kept"
    );
    let manifest = fs::read_to_string(home.state().join("manifest")).unwrap();
    assert!(manifest.contains("version 9.9.9\n"), "{manifest}");
    assert!(
        manifest.contains(&format!("previous {ours}\n")),
        "{manifest}"
    );
    // Nothing is left beside the program.
    let beside: Vec<_> = fs::read_dir(home.command().parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(beside, ["nvmimic"]);

    let out = with_release(&home, &release, &["self", "update", "--force"]);
    assert!(text(&out.stdout).contains("is the newest version"));

    let out = with_release(&home, &release, &["self", "rollback", "--force"]);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    assert_eq!(fs::read(home.command()).unwrap(), original);
    let manifest = fs::read_to_string(home.state().join("manifest")).unwrap();
    assert!(
        manifest.contains(&format!("version {ours}\n")),
        "{manifest}"
    );
    assert!(manifest.contains("previous 9.9.9\n"), "{manifest}");
    assert!(!kept.exists());
    let nvm = home.shell("nvm --version");
    assert_eq!(text(&nvm.stdout), "0.40.8\n", "{}", text(&nvm.stderr));

    let out = home.nvmimic(&["self", "uninstall", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!home.has_own_files());
}

#[cfg(target_os = "macos")]
#[test]
fn an_update_that_cannot_set_itself_up_is_undone() {
    let home = Home::new();
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let original = fs::read(home.command()).unwrap();
    let manifest = fs::read_to_string(home.state().join("manifest")).unwrap();

    let release = fake_release(&home, "9.9.9", 1);
    let out = with_release(&home, &release, &["self", "update", "--force"]);
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("was put back"),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(fs::read(home.command()).unwrap(), original);
    assert_eq!(
        fs::read_to_string(home.state().join("manifest")).unwrap(),
        manifest
    );
    let nvm = home.shell("nvm --version");
    assert_eq!(text(&nvm.stdout), "0.40.8\n", "{}", text(&nvm.stderr));
}

#[cfg(target_os = "macos")]
#[test]
fn a_download_that_fails_its_checks_changes_nothing() {
    let home = Home::new();
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let original = fs::read(home.command()).unwrap();

    let release = fake_release(&home, "9.9.9", 0);
    fs::write(
        release.join("SHA256SUMS"),
        "0000  nvmimic-darwin-universal.tar.gz\n",
    )
    .unwrap();
    let out = with_release(&home, &release, &["self", "update", "--force"]);
    assert!(text(&out.stderr).contains("does not match its checksum"));

    // The archive says 9.9.9 and the release says 9.9.10.
    let release = fake_release(&home, "9.9.9", 0);
    fs::write(release.join("VERSION"), "9.9.10\n").unwrap();
    let out = with_release(&home, &release, &["self", "update", "--force"]);
    assert!(text(&out.stderr).contains("does not run as nvmimic 9.9.10"));

    // Without the switch, a program that is not a release build refuses.
    let release = fake_release(&home, "9.9.9", 0);
    let out = home.nvmimic_with(
        &["self", "update", "--force"],
        &[("NVMIMIC_RELEASES_URL", release.as_path())],
    );
    assert!(text(&out.stderr).contains("not built as a release"));

    assert_eq!(fs::read(home.command()).unwrap(), original);
    let out = home.nvmimic(&["self", "rollback", "--force"]);
    assert!(text(&out.stderr).contains("no earlier nvmimic"));
}

/// Homebrew's nvm the way it sets itself up: its scripts under the prefix,
/// links to two of them in `$NVM_DIR`, and a `brew` whose `uninstall nvm`
/// takes the scripts away. Returns the prefix.
fn homebrew_nvm(home: &Home) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let prefix = home.root.join("brew");
    let libexec = prefix.join("opt/nvm/libexec");
    fs::create_dir_all(&libexec).unwrap();
    fs::create_dir_all(prefix.join("bin")).unwrap();
    fs::create_dir_all(home.nvm_dir()).unwrap();
    for file in ["nvm.sh", "nvm-exec"] {
        fs::copy(reference().join(file), libexec.join(file)).unwrap();
        std::os::unix::fs::symlink(libexec.join(file), home.nvm_dir().join(file)).unwrap();
    }
    let brew = prefix.join("bin/brew");
    fs::write(
        &brew,
        format!(
            "#!/bin/sh\n[ \"$1 $2\" = 'uninstall nvm' ] && mv '{}' '{}'\n",
            prefix.join("opt/nvm").display(),
            prefix.join("opt/nvm-uninstalled").display()
        ),
    )
    .unwrap();
    fs::set_permissions(&brew, fs::Permissions::from_mode(0o755)).unwrap();
    prefix
}

fn is_link(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink())
}

#[test]
fn links_into_homebrew_come_back_as_files_and_never_as_dead_links() {
    for leaving in ["eject", "uninstall"] {
        let home = Home::new();
        let prefix = homebrew_nvm(&home);
        let brew: &[(&str, &Path)] = &[("HOMEBREW_PREFIX", &prefix)];
        let out = home.nvmimic_with(&["self", "install", "--force"], brew);
        assert!(out.status.success(), "{}", text(&out.stderr));
        assert!(
            !prefix.join("opt/nvm").exists(),
            "brew uninstall did not run"
        );

        let out = home.nvmimic_with(&["self", leaving, "--force"], brew);
        assert!(out.status.success(), "{leaving}: {}", text(&out.stderr));
        for file in ["nvm.sh", "nvm-exec"] {
            let path = home.nvm_dir().join(file);
            assert!(!is_link(&path), "{leaving}: {file} came back as a link");
            assert_eq!(
                fs::read(&path).unwrap(),
                fs::read(reference().join(file)).unwrap(),
                "{leaving}: {file}"
            );
        }
        let nvm = home.shell("nvm --version");
        assert_eq!(text(&nvm.stdout), "0.40.8\n", "{}", text(&nvm.stderr));
    }
}

#[test]
fn a_link_that_still_points_somewhere_comes_back_as_that_link() {
    let home = Home::new();
    let kept = home.root.join("dotfiles");
    fs::create_dir_all(&kept).unwrap();
    fs::create_dir_all(home.nvm_dir()).unwrap();
    for file in SCRIPTS {
        fs::copy(reference().join(file), kept.join(file)).unwrap();
        std::os::unix::fs::symlink(kept.join(file), home.nvm_dir().join(file)).unwrap();
    }
    let out = home.nvmimic(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    for file in SCRIPTS {
        // Writing nvmimic's script must not go through the link.
        assert!(!is_link(&home.nvm_dir().join(file)), "{file}");
        assert_eq!(
            fs::read(kept.join(file)).unwrap(),
            fs::read(reference().join(file)).unwrap(),
            "{file}"
        );
    }

    let out = home.nvmimic(&["self", "uninstall", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    for file in SCRIPTS {
        assert_eq!(
            fs::read_link(home.nvm_dir().join(file)).unwrap(),
            kept.join(file),
            "{file}"
        );
    }
}

#[test]
fn a_link_that_was_dead_before_the_install_is_never_put_back() {
    for leaving in ["eject", "uninstall"] {
        let home = Home::new();
        fs::create_dir_all(home.nvm_dir()).unwrap();
        let gone = home.root.join("gone/nvm.sh");
        std::os::unix::fs::symlink(&gone, home.nvm_dir().join("nvm.sh")).unwrap();
        let out = home.nvmimic(&["self", "install", "--force"]);
        assert!(out.status.success(), "{}", text(&out.stderr));

        let out = home.nvmimic(&["self", leaving, "--force"]);
        let said = format!("{}{}", text(&out.stdout), text(&out.stderr));
        assert!(
            said.contains(&format!(
                "nvm.sh was a link to {}, and nothing was there",
                gone.display()
            )),
            "{leaving}: {said}"
        );
        match leaving {
            // With nothing to put back, an eject would leave no `nvm` at all.
            "eject" => {
                assert_eq!(out.status.code(), Some(1), "{said}");
                assert!(
                    fs::read_to_string(home.nvm_dir().join("nvm.sh"))
                        .unwrap()
                        .contains("as nvmimic provides it")
                );
            }
            _ => {
                assert!(out.status.success(), "{said}");
                assert!(
                    fs::symlink_metadata(home.nvm_dir().join("nvm.sh")).is_err(),
                    "a dead link came back"
                );
            }
        }
    }
}

#[test]
fn a_program_a_package_manager_installed_is_said_to_stay_before_and_after() {
    let home = Home::new();
    // Homebrew keeps each version of a program under `Cellar`.
    let cellar = home.root.join("brew/Cellar/nvmimic/0.1.0/bin");
    fs::create_dir_all(&cellar).unwrap();
    let program = cellar.join("nvmimic");
    // `cp` and not `fs::copy`: Linux refuses to start a program that any
    // process still has open for writing, and another test forking at the
    // wrong moment would hold this one open.
    let copied = Command::new("cp")
        .arg(env!("CARGO_BIN_EXE_nvmimic"))
        .arg(&program)
        .status()
        .unwrap();
    assert!(copied.success());
    let run = |args: &[&str]| {
        Command::new(&program)
            .args(args)
            .env_clear()
            .env("HOME", &home.root)
            .env("PATH", "/usr/bin:/bin")
            .env("HOMEBREW_PREFIX", home.root.join("no-homebrew"))
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap()
    };
    let out = run(&["self", "install", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));

    let stays = format!(
        "leave the nvmimic program at {}, because Homebrew installed it.",
        program.display()
    );
    let dry = text(&run(&["self", "uninstall", "--dry-run"]).stdout);
    assert!(dry.contains(&format!("Would {stays}")), "{dry}");

    let out = run(&["self", "uninstall", "--force"]);
    let said = text(&out.stdout);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(
        said.contains("Done. nvmimic is out of the nvm folder."),
        "{said}"
    );
    assert!(!said.contains("nvmimic is gone"), "{said}");
    assert!(said.contains("brew uninstall nvmimic"), "{said}");
    assert!(program.is_file());
}
