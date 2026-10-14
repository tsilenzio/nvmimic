//! nvmimic and a sister tool in one nvm folder. `shelfy` (the example in
//! `crates/nvmimic/examples/shelfy.rs`) is the second program. Both are
//! installed, only one answers `nvm`, and every way of handing `nvm` from
//! one to the other is run here with the real programs, in throwaway homes.

use nvmimic::dirs::Dirs;
use nvmimic::handover::{Record, lock};
use nvmimic::identity::{Identity, program_of};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

const SCRIPTS: [&str; 3] = ["nvm.sh", "nvm-exec", "bash_completion"];
const NVMIMIC: &str = "nvmimic";
const SHELFY: &str = "shelfy";
/// The two orders the programs can arrive in.
const ORDERS: [(&str, &str); 2] = [(NVMIMIC, SHELFY), (SHELFY, NVMIMIC)];

/// The `shelfy` program, built once. `cargo test` builds examples only when
/// it runs every test of their package, so a copy from an earlier build
/// could be stale.
fn shelfy_program() -> &'static Path {
    static PROGRAM: OnceLock<PathBuf> = OnceLock::new();
    PROGRAM.get_or_init(|| {
        let mut build = Command::new(env!("CARGO"));
        build.args(["build", "--quiet", "-p", "nvmimic", "--example", "shelfy"]);
        if !cfg!(debug_assertions) {
            build.arg("--release");
        }
        assert!(build.status().unwrap().success(), "building shelfy");
        let tests = std::env::current_exe().unwrap();
        tests
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .join("examples/shelfy")
    })
}

fn program(name: &str) -> &'static Path {
    match name {
        NVMIMIC => Path::new(env!("CARGO_BIN_EXE_nvmimic")),
        _ => shelfy_program(),
    }
}

struct Home {
    root: PathBuf,
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl Home {
    /// A throwaway home. `official` puts official nvm's three scripts into
    /// `~/.nvm`, each holding its own name.
    fn new(official: bool) -> Home {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("nvmimic-sister-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".nvm")).unwrap();
        let home = Home {
            root: root.canonicalize().unwrap(),
        };
        if official {
            for name in SCRIPTS {
                fs::write(home.nvm_dir().join(name), official_text(name)).unwrap();
            }
        }
        home
    }

    fn nvm_dir(&self) -> PathBuf {
        self.root.join(".nvm")
    }

    fn state(&self, name: &str) -> PathBuf {
        self.root.join(".local/state").join(name)
    }

    fn shared(&self) -> PathBuf {
        self.state("nvmimic-shared")
    }

    /// Runs `<name> self <args>` with no terminal. Each program is told
    /// that the file being run is where it is installed, so no copy of a
    /// program is made that macOS would have to scan.
    fn run(&self, name: &str, args: &[&str]) -> Output {
        self.command(name, args).output().unwrap()
    }

    /// The command [`Home::run`] runs, not started yet.
    fn command(&self, name: &str, args: &[&str]) -> Command {
        let mut command = Command::new(program(name));
        command
            .arg("self")
            .args(args)
            .env_clear()
            .env("HOME", &self.root)
            .env("PATH", "/usr/bin:/bin")
            .env("HOMEBREW_PREFIX", self.root.join("no-homebrew"))
            .env("NVMIMIC_INSTALL_PATH", program(NVMIMIC))
            .env("SHELFY_INSTALL_PATH", program(SHELFY))
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        command
    }

    /// What `<name> init zsh` prints in this home, as a startup file with
    /// `eval "$(nvmimic init zsh)"` gets it when a shell loads.
    fn init(&self, name: &str) -> String {
        let out = Command::new(program(name))
            .args(["init", "zsh"])
            .env_clear()
            .env("HOME", &self.root)
            .env("PATH", "/usr/bin:/bin")
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out.stderr));
        assert_eq!(text(&out.stderr), "");
        text(&out.stdout)
    }

    /// Runs a `self` command with `--force` and expects it to work.
    fn ok(&self, name: &str, command: &str) -> String {
        let out = self.run(name, &[command, "--force"]);
        assert!(
            out.status.success(),
            "{name} self {command}: {}{}",
            text(&out.stdout),
            text(&out.stderr)
        );
        text(&out.stdout)
    }

    /// Who answers `nvm`: a program's name, `official`, or `nobody`.
    fn answers(&self) -> String {
        match fs::read_to_string(self.nvm_dir().join("nvm.sh")) {
            Ok(script) if script == official_text("nvm.sh") => "official".to_string(),
            Ok(script) => program_of(&script).unwrap_or("someone else").to_string(),
            Err(_) => "nobody".to_string(),
        }
    }

    fn record(&self) -> Record {
        let text = fs::read_to_string(self.shared().join("record")).unwrap_or_default();
        Record::parse(&text)
    }

    /// Whether official nvm's three scripts are in the nvm folder.
    fn official_is_back(&self) -> bool {
        SCRIPTS.iter().all(|name| {
            fs::read_to_string(self.nvm_dir().join(name)).is_ok_and(|t| t == official_text(name))
        })
    }

    /// Whether the shared backup holds official nvm's three scripts.
    fn backup_is_whole(&self) -> bool {
        SCRIPTS.iter().all(|name| {
            fs::read_to_string(self.shared().join("backup").join(name))
                .is_ok_and(|t| t == official_text(name))
        })
    }

    fn installed(&self, name: &str) -> bool {
        self.state(name).join("manifest").is_file()
    }
}

fn official_text(name: &str) -> String {
    format!("official {name}\n")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn each_takes_over_from_the_other_and_both_stay_installed() {
    for (first, second) in ORDERS {
        let home = Home::new(true);
        home.ok(first, "install");
        assert_eq!(home.answers(), first);
        assert!(home.backup_is_whole());

        // Without --force it asks, and with no terminal nothing changes.
        let asked = home.run(second, &["install"]);
        assert!(!asked.status.success());
        assert_eq!(home.answers(), first);
        assert!(!home.installed(second));

        // A dry run says what it would do.
        let dry = text(&home.run(second, &["install", "--dry-run"]).stdout);
        assert!(
            dry.contains(&format!(
                "Would put {second}'s scripts in place of {first}'s and leave {first} installed."
            )),
            "{dry}"
        );
        assert_eq!(home.answers(), first);

        let said = home.ok(second, "install");
        assert!(
            said.contains(&format!("{first} answers `nvm` in")),
            "{said}"
        );
        assert!(
            said.contains(&format!(
                "{second} and {first} are sister tools by the same"
            )),
            "{said}"
        );
        assert!(
            said.contains(&format!("Let {second} answer `nvm` from now on?")),
            "{said}"
        );
        assert_eq!(home.answers(), second);
        assert!(home.installed(first) && home.installed(second));
        let record = home.record();
        assert_eq!(record.answers.as_deref(), Some(second));
        assert_eq!(record.programs.len(), 2);
        // The sister tool's scripts were never taken for official nvm's.
        assert!(home.backup_is_whole());

        // And back again: whoever was told last is in control.
        home.ok(first, "install");
        assert_eq!(home.answers(), first);
        assert_eq!(home.record().answers.as_deref(), Some(first));
        assert!(home.installed(second));
        assert!(home.backup_is_whole());
    }
}

#[test]
fn uninstalling_the_one_in_control_hands_over_to_the_one_asleep() {
    for (first, second) in ORDERS {
        let home = Home::new(true);
        home.ok(first, "install");
        home.ok(second, "install");

        let plan = text(&home.run(second, &["uninstall", "--dry-run"]).stdout);
        assert!(
            plan.contains(&format!("Would hand `nvm` over to {first} by running")),
            "{plan}"
        );
        assert!(
            plan.contains("are sister tools by the same author"),
            "{plan}"
        );
        assert_eq!(home.answers(), second);

        let said = home.ok(second, "uninstall");
        assert!(
            said.contains(&format!("and {first} answers `nvm` now.")),
            "{said}"
        );
        assert_eq!(home.answers(), first);
        assert!(!home.state(second).exists());
        let record = home.record();
        assert_eq!(record.answers.as_deref(), Some(first));
        assert_eq!(
            record.programs.keys().collect::<Vec<_>>(),
            [first],
            "{record:?}"
        );
        assert!(home.backup_is_whole(), "the backup outlives {second}");

        // The last one out puts official nvm back.
        home.ok(first, "uninstall");
        assert!(home.official_is_back());
        assert!(!home.shared().exists());
        assert!(!home.state(first).exists());
    }
}

#[test]
fn uninstalling_the_one_asleep_leaves_the_other_in_control() {
    for (first, second) in ORDERS {
        let home = Home::new(true);
        home.ok(first, "install");
        home.ok(second, "install");
        let before = fs::read(home.nvm_dir().join("nvm.sh")).unwrap();

        let said = home.ok(first, "uninstall");
        assert!(
            said.contains(&format!("{second} answers `nvm` in")),
            "{said}"
        );
        assert!(
            said.contains(&format!("and {second} keeps answering `nvm`.")),
            "{said}"
        );
        assert_eq!(fs::read(home.nvm_dir().join("nvm.sh")).unwrap(), before);
        assert!(!home.state(first).exists());
        // The one that took over from official nvm is gone, and the backup
        // it made is still there for the other.
        assert!(home.backup_is_whole());
        assert_eq!(home.record().programs.keys().collect::<Vec<_>>(), [second]);

        home.ok(second, "uninstall");
        assert!(home.official_is_back());
        assert!(!home.shared().exists());
    }
}

#[test]
fn eject_puts_official_nvm_back_and_only_for_the_one_in_control() {
    for (first, second) in ORDERS {
        let home = Home::new(true);
        home.ok(first, "install");
        home.ok(second, "install");

        // The one asleep has no scripts in the folder to take out.
        let refused = home.run(first, &["eject", "--force"]);
        assert!(!refused.status.success());
        let said = text(&refused.stderr);
        assert!(
            said.contains(&format!("{second} answers `nvm` in"))
                && said.contains("Nothing was changed."),
            "{said}"
        );
        assert_eq!(home.answers(), second);

        // The one in control goes back to official nvm, not to its sister.
        home.ok(second, "eject");
        assert!(home.official_is_back());
        let record = home.record();
        assert_eq!(record.answers, None);
        assert!(record.programs[second].aside && !record.programs[first].aside);
        assert!(home.installed(first) && home.installed(second));

        // The sister takes over from official nvm, and its uninstall does
        // not wake the one that was told to step aside.
        home.ok(first, "install");
        assert_eq!(home.answers(), first);
        assert!(home.backup_is_whole());
        home.ok(first, "uninstall");
        assert!(home.official_is_back());
        assert!(home.installed(second));
        assert_eq!(home.record().answers, None);

        // It still takes over when told to, and leaves nothing behind.
        home.ok(second, "install");
        assert_eq!(home.answers(), second);
        home.ok(second, "uninstall");
        assert!(home.official_is_back());
        assert!(!home.shared().exists());
    }
}

#[test]
fn either_one_can_be_first_where_there_was_no_nvm() {
    for (first, second) in ORDERS {
        let home = Home::new(false);
        home.ok(first, "install");
        assert_eq!(home.answers(), first);
        home.ok(second, "install");
        assert_eq!(home.answers(), second);
        assert_eq!(home.record().backed_up, Vec::<String>::new());

        home.ok(second, "uninstall");
        assert_eq!(home.answers(), first);
        home.ok(first, "uninstall");
        // No official nvm was there before, so none of either program's
        // scripts is left. shelfy leaves the official copy it carries.
        assert_ne!(home.answers(), NVMIMIC);
        assert_ne!(home.answers(), SHELFY);
        assert!(!home.shared().exists());
    }
}

#[test]
fn disable_hands_nvm_to_the_sister_and_enable_takes_it_back() {
    let home = Home::new(true);
    home.ok(NVMIMIC, "install");
    home.ok(SHELFY, "install");

    let plan = text(&home.run(SHELFY, &["disable", "--dry-run"]).stdout);
    assert!(
        plan.contains("Would hand `nvm` over to nvmimic by running"),
        "{plan}"
    );
    let said = home.ok(SHELFY, "disable");
    assert!(said.contains("Done. nvmimic answers `nvm` in"), "{said}");
    assert_eq!(home.answers(), NVMIMIC);
    assert!(home.installed(SHELFY));
    assert!(home.record().programs[SHELFY].aside);
    // Disabled twice is said, and changes nothing.
    assert!(!home.run(SHELFY, &["disable", "--force"]).status.success());
    assert_eq!(home.answers(), NVMIMIC);

    let said = home.ok(SHELFY, "enable");
    assert!(said.contains("nvmimic answers `nvm` in"), "{said}");
    assert_eq!(home.answers(), SHELFY);
    assert!(!home.record().programs[SHELFY].aside);
    let twice = home.run(SHELFY, &["enable", "--force"]);
    assert!(
        text(&twice.stderr).contains("shelfy already answers `nvm`"),
        "{}",
        text(&twice.stderr)
    );

    // With the sister gone, disable goes to official nvm.
    home.ok(NVMIMIC, "uninstall");
    assert_eq!(home.answers(), SHELFY);
    home.ok(SHELFY, "disable");
    assert!(home.official_is_back());
    home.ok(SHELFY, "enable");
    assert_eq!(home.answers(), SHELFY);
    assert!(home.backup_is_whole());

    // A program that was disabled is not woken by its sister's uninstall.
    home.ok(NVMIMIC, "install");
    home.ok(SHELFY, "enable");
    home.ok(SHELFY, "disable");
    assert_eq!(home.answers(), NVMIMIC);
    home.ok(NVMIMIC, "uninstall");
    assert!(home.official_is_back());
    assert!(home.installed(SHELFY));

    // Enable needs an install to wake up.
    home.ok(SHELFY, "uninstall");
    let none = home.run(SHELFY, &["enable", "--force"]);
    assert!(
        text(&none.stderr).contains("shelfy is not installed yet"),
        "{}",
        text(&none.stderr)
    );
}

#[test]
fn an_edited_script_is_copied_aside_before_the_sister_writes_over_it() {
    let home = Home::new(true);
    home.ok(SHELFY, "install");
    home.ok(NVMIMIC, "install");
    let nvm_sh = home.nvm_dir().join("nvm.sh");
    let edited = format!("{}# my own line\n", fs::read_to_string(&nvm_sh).unwrap());
    fs::write(&nvm_sh, &edited).unwrap();

    home.ok(NVMIMIC, "uninstall");
    assert_eq!(home.answers(), SHELFY);
    assert_eq!(
        fs::read_to_string(home.nvm_dir().join("nvm.sh.nvmimic-saved")).unwrap(),
        edited
    );
}

#[test]
fn a_purge_takes_the_folder_and_leaves_the_sister_installed() {
    let home = Home::new(true);
    home.ok(SHELFY, "install");
    home.ok(NVMIMIC, "install");
    let plan = text(
        &home
            .run(NVMIMIC, &["uninstall", "--purge", "--dry-run"])
            .stdout,
    );
    assert!(
        plan.contains("shelfy is set up for") && plan.contains("go with the folder"),
        "{plan}"
    );

    let out = home.run(NVMIMIC, &["uninstall", "--purge", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!home.nvm_dir().exists());
    assert!(home.installed(SHELFY) && !home.installed(NVMIMIC));
    let record = home.record();
    assert_eq!(record.answers, None);
    assert_eq!(record.backed_up, Vec::<String>::new());

    // The sister sets the folder up again from scratch.
    home.ok(SHELFY, "install");
    assert_eq!(home.answers(), SHELFY);
}

/// Turns the nvmimic install in `home` into what nvmimic 0.1.0 left on
/// disk: the backup of official nvm's scripts and the lines about it in
/// nvmimic's own state folder, and no shared record.
fn as_0_1_0_left_it(home: &Home, ejected: bool) {
    let state = home.state(NVMIMIC);
    let mut manifest = fs::read_to_string(state.join("manifest")).unwrap();
    manifest = manifest.replace("ejected scripts\n", "");
    fs::create_dir_all(state.join("backup")).unwrap();
    if !ejected {
        for name in SCRIPTS {
            fs::rename(
                home.shared().join("backup").join(name),
                state.join("backup").join(name),
            )
            .unwrap();
            manifest.push_str(&format!("backup {name}\n"));
        }
    } else {
        manifest.push_str("ejected scripts\n");
    }
    let version = format!("version {}\n", env!("CARGO_PKG_VERSION"));
    manifest = manifest.replace(&version, "version 0.1.0\n");
    fs::write(state.join("manifest"), manifest).unwrap();
    fs::remove_dir_all(home.shared()).unwrap();
    fs::remove_file(state.join("shared")).unwrap();
    // 0.1.0 took no lock, so it left no file for one.
    fs::remove_file(home.state("nvmimic-shared.lock")).unwrap();
}

/// Every file under `dir` with what it holds, and every link with where it
/// points.
fn snapshot(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut found = Vec::new();
    let mut folders = vec![dir.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).into_iter().flatten().flatten() {
            let path = entry.path();
            let meta = fs::symlink_metadata(&path).unwrap();
            if meta.is_dir() {
                folders.push(path.clone());
                found.push((path, "folder".to_string()));
            } else if meta.is_symlink() {
                let target = fs::read_link(&path).unwrap();
                found.push((path, format!("link to {}", target.display())));
            } else {
                found.push((path.clone(), text(&fs::read(&path).unwrap())));
            }
        }
    }
    found.sort();
    found
}

#[test]
fn an_install_made_by_0_1_0_is_moved_into_the_shared_record_by_itself() {
    // Whichever `self` command comes first does the move.
    for command in ["install", "eject", "uninstall"] {
        let home = Home::new(true);
        home.ok(NVMIMIC, "install");
        as_0_1_0_left_it(&home, false);
        let state = home.state(NVMIMIC);
        assert!(state.join("backup/nvm.sh").is_file() && !home.shared().exists());

        home.ok(NVMIMIC, command);
        assert!(!state.join("backup").exists(), "{command}");
        match command {
            "install" => {
                assert_eq!(home.answers(), NVMIMIC);
                assert!(home.backup_is_whole());
                let record = home.record();
                assert_eq!(record.answers.as_deref(), Some(NVMIMIC));
                assert_eq!(record.backed_up, SCRIPTS);
                assert_eq!(record.nvm_dir, Some(home.nvm_dir()));
                let manifest = fs::read_to_string(state.join("manifest")).unwrap();
                assert!(!manifest.contains("backup "), "{manifest}");
                // And the sister tool can join it from here.
                home.ok(SHELFY, "install");
                assert_eq!(home.answers(), SHELFY);
                home.ok(SHELFY, "uninstall");
                home.ok(NVMIMIC, "uninstall");
                assert!(home.official_is_back());
            }
            "eject" => {
                assert!(home.official_is_back());
                assert!(home.record().programs[NVMIMIC].aside);
            }
            _ => {
                assert!(home.official_is_back());
                assert!(!home.shared().exists() && !state.exists());
            }
        }
    }
}

#[test]
fn a_dry_run_on_an_install_made_by_0_1_0_moves_nothing() {
    let home = Home::new(true);
    home.ok(NVMIMIC, "install");
    as_0_1_0_left_it(&home, false);
    let before = snapshot(&home.root);

    // The plan is worked out from the records where 0.1.0 left them.
    let plan = text(&home.run(NVMIMIC, &["uninstall", "--dry-run"]).stdout);
    assert!(
        plan.contains("Would put nvm's own nvm.sh, nvm-exec, bash_completion back in"),
        "{plan}"
    );
    assert_eq!(snapshot(&home.root), before, "uninstall --dry-run");
    let plan = text(&home.run(NVMIMIC, &["eject", "--dry-run"]).stdout);
    assert!(plan.contains("Would put nvm's own nvm.sh"), "{plan}");
    assert_eq!(snapshot(&home.root), before, "eject --dry-run");
    assert!(
        home.run(NVMIMIC, &["install", "--dry-run"])
            .status
            .success()
    );
    assert_eq!(snapshot(&home.root), before, "install --dry-run");
    // A command that stops for lack of a terminal moves nothing either.
    assert!(!home.run(NVMIMIC, &["uninstall"]).status.success());
    assert_eq!(snapshot(&home.root), before, "uninstall with no terminal");

    home.ok(NVMIMIC, "uninstall");
    assert!(home.official_is_back());
    assert!(!home.shared().exists() && !home.state(NVMIMIC).exists());
}

#[test]
fn an_ejected_install_made_by_0_1_0_joins_as_one_that_stepped_aside() {
    let home = Home::new(true);
    home.ok(NVMIMIC, "install");
    home.ok(NVMIMIC, "eject");
    as_0_1_0_left_it(&home, true);

    // shelfy arrives first and knows nothing of nvmimic yet.
    home.ok(SHELFY, "install");
    assert_eq!(home.record().programs.len(), 1);
    // A command of nvmimic's that is refused changes nothing.
    let refused = home.run(NVMIMIC, &["eject", "--force"]);
    assert!(!refused.status.success());
    assert_eq!(home.record().programs.len(), 1);
    assert_eq!(home.answers(), SHELFY);

    // Its uninstall is not handed to nvmimic, which nobody told to wake.
    home.ok(SHELFY, "uninstall");
    assert!(home.official_is_back());
    assert!(home.installed(NVMIMIC));

    // nvmimic's next `self` command that goes ahead brings it in.
    home.ok(NVMIMIC, "install");
    let record = home.record();
    assert_eq!(record.answers.as_deref(), Some(NVMIMIC));
    assert!(!record.programs[NVMIMIC].aside, "{record:?}");
    home.ok(NVMIMIC, "uninstall");
    assert!(home.official_is_back());
}

#[test]
fn a_sister_from_before_the_shared_record_is_not_written_over() {
    let home = Home::new(true);
    home.ok(NVMIMIC, "install");
    as_0_1_0_left_it(&home, false);
    let before = fs::read(home.nvm_dir().join("nvm.sh")).unwrap();

    let refused = home.run(SHELFY, &["install", "--force"]);
    assert!(!refused.status.success());
    let said = text(&refused.stderr);
    assert!(
        said.contains("nvmimic's scripts are in")
            && said.contains("Update nvmimic and run `nvmimic self install`")
            && said.contains("Nothing was changed."),
        "{said}"
    );
    assert_eq!(fs::read(home.nvm_dir().join("nvm.sh")).unwrap(), before);
    assert!(!home.installed(SHELFY) && !home.shared().exists());

    // What the message says to do works.
    home.ok(NVMIMIC, "install");
    home.ok(SHELFY, "install");
    assert_eq!(home.answers(), SHELFY);
    assert!(home.backup_is_whole());
}

#[test]
fn each_program_finds_the_shared_folder_through_its_own_link() {
    let home = Home::new(true);
    home.ok(NVMIMIC, "install");
    home.ok(SHELFY, "install");
    for name in [NVMIMIC, SHELFY] {
        assert_eq!(
            fs::read_link(home.state(name).join("shared")).unwrap(),
            home.shared()
        );
    }
    let readme = fs::read_to_string(home.shared().join("README")).unwrap();
    assert!(
        readme.contains("nvmimic and nvmise") && readme.contains("don't delete"),
        "{readme}"
    );

    // The folder is renamed by moving it and pointing each link at it.
    let moved = home.root.join("moved/shared-by-the-two");
    fs::create_dir_all(moved.parent().unwrap()).unwrap();
    fs::rename(home.shared(), &moved).unwrap();
    for name in [NVMIMIC, SHELFY] {
        let link = home.state(name).join("shared");
        fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink(&moved, &link).unwrap();
    }
    home.ok(NVMIMIC, "install");
    assert_eq!(home.answers(), NVMIMIC);
    assert!(
        !home.shared().exists(),
        "nothing went to the standard place"
    );
    assert!(moved.join("record").is_file());

    // Each program takes its own link along, and the last one the folder.
    home.ok(NVMIMIC, "uninstall");
    assert_eq!(home.answers(), SHELFY);
    assert!(!home.state(NVMIMIC).exists());
    assert!(moved.join("README").is_file());
    home.ok(SHELFY, "uninstall");
    assert!(home.official_is_back());
    assert!(!moved.exists() && !home.state(SHELFY).exists());
}

#[test]
fn a_deleted_link_and_one_pointing_nowhere_are_put_back() {
    for broken in ["deleted", "pointing nowhere"] {
        for name in [NVMIMIC, SHELFY] {
            let home = Home::new(true);
            home.ok(name, "install");
            let link = home.state(name).join("shared");
            fs::remove_file(&link).unwrap();
            if broken == "pointing nowhere" {
                std::os::unix::fs::symlink(home.root.join("not/there"), &link).unwrap();
            }

            // The standard place is used, and nothing is said to be lost.
            let plan = text(&home.run(name, &["eject", "--dry-run"]).stdout);
            assert!(plan.contains("Would put nvm's own nvm.sh"), "{plan}");
            assert!(!plan.contains("no backup to restore"), "{plan}");
            home.ok(name, "eject");
            assert!(home.official_is_back(), "{name}, link {broken}");
            assert_eq!(fs::read_link(&link).unwrap(), home.shared());
        }
    }
}

#[test]
fn with_the_shared_folder_deleted_both_carry_on_and_say_there_is_no_backup() {
    for (first, second) in ORDERS {
        let home = Home::new(true);
        home.ok(first, "install");
        home.ok(second, "install");
        fs::remove_dir_all(home.shared()).unwrap();

        // Eject has nothing to go back to and says why. shelfy carries a
        // copy of official nvm for that, so only nvmimic is asked.
        if second == NVMIMIC {
            let refused = home.run(second, &["eject", "--force"]);
            assert!(!refused.status.success());
            let said = text(&refused.stderr);
            assert!(
                said.contains("was deleted")
                    && said.contains("so there is no backup to restore. Nothing was changed."),
                "{said}"
            );
        }
        assert_eq!(home.answers(), second);

        // The one asleep still takes over, and the other still hands back.
        home.ok(first, "install");
        assert_eq!(home.answers(), first);
        home.ok(second, "install");
        assert_eq!(home.answers(), second);
        assert!(home.shared().join("README").is_file());

        // Both uninstall, and each says there is no backup to restore.
        let said = home.ok(second, "uninstall");
        assert!(said.contains("so there is no backup to restore."), "{said}");
        assert!(!home.state(second).exists());
        let said = home.ok(first, "uninstall");
        assert!(said.contains("so there is no backup to restore."), "{said}");
        assert!(!home.state(first).exists());
        assert!(!home.shared().exists());
        assert!(!home.official_is_back());
    }
}

/// An install that nvmimic 0.1.0 made, after Homebrew or anything else but
/// `self update` put a newer nvmimic in its place: no `self` command ran.
#[test]
fn an_upgraded_nvmimic_joins_the_shared_record_when_a_shell_loads_it() {
    for ejected in [false, true] {
        let home = Home::new(true);
        home.ok(NVMIMIC, "install");
        if ejected {
            home.ok(NVMIMIC, "eject");
        }
        let printed = home.init(NVMIMIC);
        as_0_1_0_left_it(&home, ejected);
        let manifest = || fs::read_to_string(home.state(NVMIMIC).join("manifest")).unwrap();
        assert!(manifest().contains("version 0.1.0\n"));

        // While another command holds the lock, the shell loads as always
        // and nothing moves.
        let root = home.root.to_string_lossy().into_owned();
        let dirs = Dirs::resolve(|name| (name == "HOME").then(|| root.clone())).unwrap();
        let held = lock(&dirs, &Identity::NVMIMIC, Duration::ZERO).unwrap();
        let before = snapshot(&home.root);
        assert_eq!(home.init(NVMIMIC), printed);
        assert_eq!(snapshot(&home.root), before);
        drop(held);

        // The first shell to load moves the install, and prints what a
        // shell always gets.
        assert_eq!(home.init(NVMIMIC), printed);
        let record = home.record();
        assert_eq!(record.programs[NVMIMIC].aside, ejected);
        assert_eq!(record.answers.as_deref(), (!ejected).then_some(NVMIMIC));
        assert_eq!(home.backup_is_whole(), !ejected);
        assert!(!home.state(NVMIMIC).join("backup").exists());
        let now = manifest();
        let version = format!("version {}\n", env!("CARGO_PKG_VERSION"));
        assert!(
            now.contains(&version) && !now.contains("\nbackup "),
            "{now}"
        );
        assert_eq!(now.contains("ejected scripts\n"), ejected, "{now}");

        // Every shell after that changes nothing.
        let after = snapshot(&home.root);
        assert_eq!(home.init(NVMIMIC), printed);
        assert_eq!(snapshot(&home.root), after);

        // The sister tool is not refused any more, and both leave cleanly.
        home.ok(SHELFY, "install");
        assert_eq!(home.answers(), SHELFY);
        home.ok(SHELFY, "uninstall");
        assert_eq!(home.answers(), if ejected { "official" } else { NVMIMIC });
        home.ok(NVMIMIC, "uninstall");
        assert!(home.official_is_back());
        assert!(!home.shared().exists() && !home.state(NVMIMIC).exists());
    }
}

/// Every hand-over runs the sister's `self install --force`, and `--force`
/// takes the default of a question about something extra. So no program
/// ever makes a copy of the whole nvm folder because it was handed `nvm`.
#[test]
fn no_forced_command_and_no_hand_over_copies_the_nvm_folder() {
    for (first, second) in ORDERS {
        let home = Home::new(true);
        let mut said = String::new();
        for (name, command) in [
            (first, "install"),
            (second, "install"),
            // Hands `nvm` back to the first.
            (second, "uninstall"),
            (second, "install"),
            (first, "uninstall"),
            (second, "eject"),
            (second, "install"),
            (second, "uninstall"),
        ] {
            said.push_str(&home.ok(name, command));
            let copies: Vec<_> = fs::read_dir(&home.root)
                .unwrap()
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .filter(|file| file.contains("backup"))
                .collect();
            assert!(copies.is_empty(), "{name} self {command}: {copies:?}");
        }
        assert!(home.official_is_back());
        // The first install over official nvm is the one that has the
        // question, and it is answered with its default.
        assert_eq!(said.matches("to a backup folder first?").count(), 1);
        assert!(said.contains("to a backup folder first? [y/N] n (--force)"));
        assert!(!said.contains("Copied "), "{said}");
    }
}

/// nvmimic uses the Node versions and aliases in the nvm folder, and says
/// so. A program with a store of its own says what is true for it, in the
/// same places.
#[test]
fn each_program_says_what_is_true_for_it_about_the_nvm_folder() {
    let ours = [
        "nvmimic aims to be a drop-in replacement: it uses the Node versions, aliases\n\
         and settings that are already there, and only swaps out nvm's scripts.\n",
        "Your Node versions and aliases stay as they are.\n",
        "so `nvmimic self install` can take over again.\n",
    ];
    let home = Home::new(true);
    let said = home.ok(NVMIMIC, "install") + &home.ok(NVMIMIC, "eject");
    for sentence in ours {
        assert!(said.contains(sentence), "{sentence}: {said}");
    }
    let said = home.ok(NVMIMIC, "uninstall");
    assert!(said.contains(ours[1]), "{said}");

    let home = Home::new(true);
    let said = home.ok(SHELFY, "install") + &home.ok(SHELFY, "eject");
    assert!(
        said.contains(
            "shelfy keeps Node versions and aliases on a shelf of its own and uses none of\n\
             the ones in the nvm folder. It only swaps out nvm's scripts.\n"
        ),
        "{said}"
    );
    assert!(
        said.contains("so `shelfy self enable` can take over again.\n"),
        "{said}"
    );
    let said = said + &home.ok(SHELFY, "uninstall");
    for word in ["drop-in", "stay as they are", "shelfy self install` can"] {
        assert!(!said.contains(word), "{word}: {said}");
    }
    assert!(home.official_is_back());
}

#[test]
fn a_self_command_waits_for_the_lock_and_then_says_who_holds_it() {
    let home = Home::new(true);
    home.ok(NVMIMIC, "install");
    home.ok(SHELFY, "install");
    let root = home.root.to_string_lossy().into_owned();
    let dirs = Dirs::resolve(|name| (name == "HOME").then(|| root.clone())).unwrap();
    assert_eq!(dirs.shared_lock(), home.state("nvmimic-shared.lock"));

    let held = lock(&dirs, &Identity::NVMIMIC, Duration::ZERO).unwrap();
    let before = snapshot(&home.root);
    let holder = format!("(nvmimic, process {})", std::process::id());
    for (name, command) in [
        (NVMIMIC, "install"),
        (NVMIMIC, "eject"),
        (NVMIMIC, "uninstall"),
        (SHELFY, "uninstall"),
        (SHELFY, "disable"),
    ] {
        let out = home.run(name, &[command, "--force"]);
        let said = text(&out.stderr);
        assert_eq!(out.status.code(), Some(1), "{name} self {command}: {said}");
        assert!(said.contains(&holder), "{name} self {command}: {said}");
        assert!(said.contains("Nothing was changed."), "{said}");
        assert_eq!(snapshot(&home.root), before, "{name} self {command}");
    }
    // A dry run changes nothing, so it waits for nobody.
    assert!(
        home.run(NVMIMIC, &["uninstall", "--dry-run"])
            .status
            .success()
    );

    drop(held);
    // A hand-over runs the sister's `self install` under the lock of the
    // command that started it.
    home.ok(SHELFY, "uninstall");
    assert_eq!(home.answers(), NVMIMIC);
    home.ok(NVMIMIC, "uninstall");
    assert!(home.official_is_back());
    assert!(!home.shared().exists() && !dirs.shared_lock().exists());
}

#[test]
fn two_self_commands_at_once_leave_one_record_that_matches_the_folder() {
    for round in 0..6 {
        let home = Home::new(true);
        // Each program starts three commands, all at the same moment.
        let commands = ["install", "install", "install"];
        let started: Vec<(&str, std::process::Child)> = commands
            .iter()
            .flat_map(|command| [NVMIMIC, SHELFY].map(|name| (name, *command)))
            .map(|(name, command)| (name, home.command(name, &[command, "--force"])))
            .map(|(name, mut command)| (name, command.spawn().unwrap()))
            .collect();
        for (name, child) in started {
            let out = child.wait_with_output().unwrap();
            let said = text(&out.stderr);
            // A command either went through or found the lock taken.
            assert!(
                out.status.success() || said.contains("another `self` command is still running"),
                "round {round}, {name}: {said}"
            );
        }
        let record = home.record();
        let answers = home.answers();
        assert!(
            answers == NVMIMIC || answers == SHELFY,
            "round {round}: {answers}"
        );
        assert_eq!(record.answers.as_deref(), Some(answers.as_str()));
        assert!(home.backup_is_whole(), "round {round}");
        // Without the lock, two installs each read a record without the
        // other and the later write drops the earlier program.
        for name in [NVMIMIC, SHELFY] {
            assert_eq!(
                record.programs.contains_key(name),
                home.installed(name),
                "round {round}: {name} in {record:?}"
            );
        }
        // Whatever order they ran in, both leave again and nvm is back.
        for name in [NVMIMIC, SHELFY] {
            if home.installed(name) {
                home.ok(name, "uninstall");
            }
        }
        assert!(home.official_is_back(), "round {round}");
        assert!(!home.shared().exists(), "round {round}");
    }
}

/// The released nvmimic 0.1.0, if this machine has it: the program that
/// `NVMIMIC_RELEASED_0_1_0` names, or the one Homebrew installed.
fn released_0_1_0() -> Option<PathBuf> {
    let program = std::env::var_os("NVMIMIC_RELEASED_0_1_0")
        .map_or_else(|| PathBuf::from("/opt/homebrew/bin/nvmimic"), PathBuf::from);
    let version = Command::new(&program).arg("--version").output().ok()?;
    // With Homebrew's nvm installed, `self install` would uninstall it.
    let homebrew_nvm = Path::new("/opt/homebrew/opt/nvm").exists();
    (text(&version.stdout).trim() == "0.1.0" && !homebrew_nvm).then_some(program)
}

/// A home with official nvm 0.40.8's own three scripts, taken over by the
/// released nvmimic 0.1.0.
fn home_set_up_by_0_1_0(released: &Path) -> Home {
    let home = Home::new(false);
    let reference = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/reference/nvm-v0.40.8");
    for name in SCRIPTS {
        fs::copy(reference.join(name), home.nvm_dir().join(name)).unwrap();
    }
    let installed = Command::new(released)
        .args(["self", "install", "--force"])
        .env_clear()
        .env("HOME", &home.root)
        .env("PATH", "/usr/bin:/bin")
        .env("HOMEBREW_PREFIX", home.root.join("no-homebrew"))
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(installed.status.success(), "{}", text(&installed.stderr));
    home
}

#[test]
fn an_install_by_the_released_0_1_0_is_moved_ejected_and_uninstalled() {
    let Some(released) = released_0_1_0() else {
        eprintln!(
            "skipped: nvmimic 0.1.0 is not at /opt/homebrew/bin/nvmimic, and NVMIMIC_RELEASED_0_1_0 names none"
        );
        return;
    };
    let reference = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/reference/nvm-v0.40.8");
    let official = |name: &str| fs::read(reference.join(name)).unwrap();
    let is_official = |home: &Home| {
        SCRIPTS
            .iter()
            .all(|name| fs::read(home.nvm_dir().join(name)).is_ok_and(|b| b == official(name)))
    };

    let home = home_set_up_by_0_1_0(&released);
    let state = home.state(NVMIMIC);
    let manifest = fs::read_to_string(state.join("manifest")).unwrap();
    assert!(manifest.contains("version 0.1.0\n"), "{manifest}");
    for name in SCRIPTS {
        assert!(manifest.contains(&format!("backup {name}\n")), "{manifest}");
        assert_eq!(
            fs::read(state.join("backup").join(name)).unwrap(),
            official(name)
        );
    }
    assert_eq!(home.answers(), NVMIMIC);
    assert!(!home.shared().exists());

    // A dry run by this build moves nothing.
    let before = snapshot(&home.root);
    let plan = text(&home.run(NVMIMIC, &["uninstall", "--dry-run"]).stdout);
    assert!(
        plan.contains("Would put nvm's own nvm.sh, nvm-exec, bash_completion back in"),
        "{plan}"
    );
    assert_eq!(snapshot(&home.root), before);

    // This build over it: the move.
    home.ok(NVMIMIC, "install");
    assert!(!state.join("backup").exists());
    let manifest = fs::read_to_string(state.join("manifest")).unwrap();
    assert!(!manifest.contains("backup "), "{manifest}");
    let record = home.record();
    assert_eq!(record.backed_up, SCRIPTS);
    assert_eq!(record.answers.as_deref(), Some(NVMIMIC));
    assert!(!record.lost);
    for name in SCRIPTS {
        assert_eq!(
            fs::read(home.shared().join("backup").join(name)).unwrap(),
            official(name)
        );
    }
    assert_eq!(fs::read_link(state.join("shared")).unwrap(), home.shared());
    assert_eq!(home.answers(), NVMIMIC);

    // An eject, then taking over again, then an uninstall.
    home.ok(NVMIMIC, "eject");
    assert!(is_official(&home));
    home.ok(NVMIMIC, "install");
    assert_eq!(home.answers(), NVMIMIC);
    home.ok(NVMIMIC, "uninstall");
    assert!(is_official(&home));
    assert!(!home.shared().exists() && !state.exists());

    // And each of the two straight over what 0.1.0 left.
    for command in ["eject", "uninstall"] {
        let home = home_set_up_by_0_1_0(&released);
        home.ok(NVMIMIC, command);
        assert!(is_official(&home), "{command}");
        assert!(!home.state(NVMIMIC).join("backup").exists(), "{command}");
        assert_eq!(home.shared().exists(), command == "eject");
    }

    // And a shell that loads this build over what 0.1.0 left, which is
    // what happens after `brew upgrade`.
    let home = home_set_up_by_0_1_0(&released);
    home.init(NVMIMIC);
    let state = home.state(NVMIMIC);
    assert!(!state.join("backup").exists());
    let manifest = fs::read_to_string(state.join("manifest")).unwrap();
    let version = format!("version {}\n", env!("CARGO_PKG_VERSION"));
    assert!(
        manifest.contains(&version) && !manifest.contains("backup "),
        "{manifest}"
    );
    let record = home.record();
    assert_eq!(record.backed_up, SCRIPTS);
    assert_eq!(record.answers.as_deref(), Some(NVMIMIC));
    for name in SCRIPTS {
        assert_eq!(
            fs::read(home.shared().join("backup").join(name)).unwrap(),
            official(name)
        );
    }
    home.ok(NVMIMIC, "uninstall");
    assert!(is_official(&home));
    assert!(!home.shared().exists() && !state.exists());
}
