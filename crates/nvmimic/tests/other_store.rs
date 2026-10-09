//! A second backend under a second identity, to show that the library needs
//! neither nvm's own folder layout nor nvmimic's own name. `Shelf` (in
//! `common/shelf.rs`) keeps every version in one folder outside `$NVM_DIR`,
//! names the folders without the leading `v`, keeps its aliases in a folder
//! of its own, and installs versions itself. `shelfy` is the program built
//! on it, in `examples/shelfy.rs`. The tests here run nvm's commands
//! through [`nvmimic::run`], the installer through `nvmimic::install`, and
//! real shells through the `shelfy` program, the way another program would.

#[path = "common/shelf.rs"]
mod shelf;

use nvmimic::context::CallerShell;
use nvmimic::shell::{Dialect, Effect, Vars};
use nvmimic::{Caller, Outcome};
use shelf::{CARRIED_NVM_SH, SHELFY, Shelf, ShelfSteps, fake_node};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

/// A throwaway home with an empty `$NVM_DIR` and a shelf beside it.
struct World {
    root: PathBuf,
    shelf: Shelf,
    vars: BTreeMap<String, String>,
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct Ran {
    out: String,
    err: String,
    outcome: Outcome,
}

impl World {
    fn new() -> World {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir()
            .join(format!("nvmimic-shelf-{}-{id}", std::process::id()))
            .canonicalize_after_create();
        let shelf = Shelf::at(&root);
        // An empty folder as `PATH`, so a Node installed on this machine
        // cannot show up as nvm's `system`.
        let empty = root.join("empty");
        for dir in [&shelf.store, &shelf.aliases, &root.join(".nvm"), &empty] {
            fs::create_dir_all(dir).unwrap();
        }
        let vars = [
            ("HOME", root.display().to_string()),
            ("NVM_DIR", root.join(".nvm").display().to_string()),
            ("PATH", empty.display().to_string()),
            ("PWD", root.display().to_string()),
        ]
        .into_iter()
        .map(|(name, value)| (name.to_string(), value))
        .collect();
        World { root, shelf, vars }
    }

    fn with(&self, versions: &[&str]) -> &World {
        for version in versions {
            fake_node(&self.shelf.folder(version), version);
        }
        self
    }

    fn nvm(&self, args: &[&str]) -> Ran {
        self.call("nvm", args)
    }

    fn call(&self, function: &str, args: &[&str]) -> Ran {
        let caller = Caller {
            vars: Vars::from_env(self.vars.clone()),
            dialect: Dialect::Bash,
            stdout_is_tty: false,
            aliases: Vec::new(),
            command_info: String::new(),
            shell: CallerShell::default(),
            direct_output: false,
            cache_dir: None,
            identity: &SHELFY,
        };
        let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let outcome = nvmimic::run(
            caller,
            &self.shelf,
            function,
            &args,
            &mut out,
            &mut err,
            &mut |_| false,
        );
        Ran {
            out: String::from_utf8(out).unwrap(),
            err: String::from_utf8(err).unwrap(),
            outcome,
        }
    }

    /// Carries a call's changes to the variables over to the next call, as
    /// the shell would.
    fn apply(&mut self, ran: &Ran) {
        for effect in &ran.outcome.effects {
            match effect {
                Effect::Set { name, value, .. } => {
                    self.vars.insert(name.clone(), value.clone());
                }
                Effect::Unset(name) => {
                    self.vars.remove(name);
                }
                _ => {}
            }
        }
    }

    fn empty(&self) -> String {
        self.root.join("empty").display().to_string()
    }

    fn path_of(&self, version: &str) -> String {
        self.shelf.folder(version).display().to_string()
    }

    /// Nothing of a version's may show up in nvm's own layout.
    fn assert_nvm_dir_untouched(&self) {
        let nvm_dir = self.root.join(".nvm");
        assert!(!nvm_dir.join("versions").exists());
        assert!(!nvm_dir.join("alias").exists());
    }
}

trait CanonicalizeAfterCreate {
    fn canonicalize_after_create(self) -> PathBuf;
}

impl CanonicalizeAfterCreate for PathBuf {
    /// macOS's temp folder is behind a symlink, and nvm compares paths as
    /// text, so the tests use the real path throughout.
    fn canonicalize_after_create(self) -> PathBuf {
        let _ = fs::remove_dir_all(&self);
        fs::create_dir_all(&self).unwrap();
        self.canonicalize().unwrap()
    }
}

/// Answers every request with a release list holding one version, and
/// returns the address to use as nvm's mirror.
fn serve_index() -> String {
    const INDEX: &str = "version\tdate\tfiles\tnpm\tv8\tuv\tzlib\topenssl\tmodules\tlts\tsecurity\n\
        v22.1.0\t2024-05-02\tosx-arm64-tar,linux-x64,linux-arm64\t10.7.0\t12.4\t1.48.0\t1.3.0\t3.0.13\t127\t-\t-\n\
        v20.11.1\t2024-02-13\tosx-arm64-tar,linux-x64,linux-arm64\t10.2.4\t11.3\t1.46.0\t1.2.13\t3.0.13\t115\tIron\t-\n";
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut request = [0; 2048];
            let _ = stream.read(&mut request);
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{INDEX}",
                INDEX.len()
            );
        }
    });
    address
}

#[test]
fn lists_and_resolves_versions_from_the_shelf() {
    let world = World::new();
    world.with(&["v20.11.1", "v22.1.0", "v22.3.0"]);

    let ran = world.call("nvm_ls", &[]);
    assert_eq!(ran.out, "v20.11.1\nv22.1.0\nv22.3.0\n");

    let ran = world.call("nvm_version", &["22"]);
    assert_eq!((ran.out.as_str(), ran.outcome.status), ("v22.3.0\n", 0));

    let ran = world.call("nvm_version", &["18"]);
    assert_eq!((ran.out.as_str(), ran.outcome.status), ("N/A\n", 3));

    let ran = world.call("nvm_version_path", &["v22.1.0"]);
    assert_eq!(ran.out, format!("{}\n", world.path_of("v22.1.0")));

    let ran = world.call("nvm_is_version_installed", &["v20.11.1"]);
    assert_eq!(ran.outcome.status, 0);
    let ran = world.call("nvm_is_version_installed", &["v20.11.2"]);
    assert_ne!(ran.outcome.status, 0);
}

#[test]
fn which_points_into_the_shelf() {
    let world = World::new();
    world.with(&["v22.1.0"]);
    let ran = world.nvm(&["which", "22"]);
    assert_eq!(ran.out, format!("{}/bin/node\n", world.path_of("v22.1.0")));
    assert_eq!(ran.outcome.status, 0);
}

#[test]
fn use_current_and_deactivate_follow_the_shelf() {
    let mut world = World::new();
    world.with(&["v20.11.1", "v22.1.0"]);

    let ran = world.nvm(&["current"]);
    assert_eq!(ran.out, "none\n");

    let ran = world.nvm(&["use", "22"]);
    assert_eq!(ran.outcome.status, 0, "{}", ran.err);
    assert!(ran.out.starts_with("Now using node v22.1.0"), "{}", ran.out);
    world.apply(&ran);
    let bin = format!("{}/bin", world.path_of("v22.1.0"));
    assert_eq!(world.vars["PATH"], format!("{bin}:{}", world.empty()));
    assert_eq!(world.vars["NVM_BIN"], bin);

    let ran = world.nvm(&["current"]);
    assert_eq!(ran.out, "v22.1.0\n");

    // A second `use` swaps the entry in place instead of stacking another.
    let ran = world.nvm(&["use", "20"]);
    world.apply(&ran);
    assert_eq!(
        world.vars["PATH"],
        format!("{}/bin:{}", world.path_of("v20.11.1"), world.empty())
    );

    let ran = world.nvm(&["deactivate"]);
    assert_eq!(ran.outcome.status, 0);
    world.apply(&ran);
    assert_eq!(world.vars["PATH"], world.empty());
    assert!(!world.vars.contains_key("NVM_BIN"));
    world.assert_nvm_dir_untouched();
}

#[test]
fn aliases_live_in_the_shelf_s_own_folder() {
    let world = World::new();
    world.with(&["v22.1.0"]);

    let ran = world.nvm(&["alias", "default", "22"]);
    assert_eq!(ran.outcome.status, 0, "{}", ran.err);
    assert_eq!(
        fs::read_to_string(world.shelf.aliases.join("default")).unwrap(),
        "22\n"
    );

    let ran = world.call("nvm_version", &["default"]);
    assert_eq!(ran.out, "v22.1.0\n");
    world.assert_nvm_dir_untouched();
}

#[test]
fn uninstall_removes_the_version_from_the_shelf() {
    let world = World::new();
    world.with(&["v20.11.1", "v22.1.0"]);
    fs::write(world.shelf.aliases.join("old"), "v20.11.1\n").unwrap();

    let ran = world.nvm(&["uninstall", "20"]);
    assert_eq!(ran.outcome.status, 0, "{}", ran.err);
    assert!(
        ran.out.starts_with("Uninstalled node v20.11.1\n"),
        "{}",
        ran.out
    );
    assert!(!world.shelf.folder("v20.11.1").exists());
    assert!(world.shelf.folder("v22.1.0").exists());
    assert!(!world.shelf.aliases.join("old").exists());
}

#[test]
fn install_goes_through_the_shelf() {
    let mut world = World::new();
    // nvm asks the mirror which versions exist before it installs one, and
    // curl fetches the list.
    world
        .vars
        .insert("PATH".to_string(), "/usr/bin:/bin".to_string());
    world
        .vars
        .insert("NVM_NODEJS_ORG_MIRROR".to_string(), serve_index());

    let ran = world.nvm(&["install", "22"]);
    assert_eq!(ran.outcome.status, 0, "{}{}", ran.out, ran.err);
    assert!(
        ran.out.contains("shelf: putting v22.1.0 in place\n"),
        "{}",
        ran.out
    );
    assert!(ran.out.contains("Now using node v22.1.0"), "{}", ran.out);
    world.apply(&ran);
    assert!(world.shelf.folder("v22.1.0").join("bin/node").exists());
    assert_eq!(
        world.vars["PATH"],
        format!("{}/bin:/usr/bin:/bin", world.path_of("v22.1.0"))
    );
    assert!(!world.root.join(".nvm/versions").exists());

    let ran = world.nvm(&["install", "22"]);
    assert!(
        ran.err.contains("v22.1.0 is already installed."),
        "{}",
        ran.err
    );
}

#[test]
fn completion_leaves_out_what_the_shelf_cannot_do() {
    let world = World::new();
    world.with(&["v20.11.1", "v22.3.0"]);

    // The shelf never builds from source, so `-s` isn't offered.
    let ran = world.call("__nvm_complete", &["bash", "2", "nvm", "install", "-"]);
    let flags: Vec<&str> = ran.out.lines().collect();
    assert!(!flags.contains(&"-s"), "{flags:?}");
    assert!(flags.contains(&"-b"), "{flags:?}");
    assert!(flags.contains(&"--lts"), "{flags:?}");

    // Versions come from the shelf, not from `$NVM_DIR/versions`.
    let ran = world.call("__nvm_complete", &["bash", "2", "nvm", "use", "v2"]);
    assert_eq!(ran.out, "v20.11.1\nv22.3.0\n");

    // zsh gets a few words about each entry.
    let ran = world.call("__nvm_complete", &["zsh", "1", "nvm", "unl"]);
    assert_eq!(ran.out, "unload:remove nvm from this shell\n");
}

#[test]
fn the_shelf_hears_about_every_switch() {
    let mut world = World::new();
    world.with(&["v20.11.1", "v22.1.0"]);

    let ran = world.nvm(&["use", "22"]);
    assert!(ran.outcome.effects.contains(&Effect::Set {
        name: "SHELF_NODE_VERSION".to_string(),
        value: "22.1.0".to_string(),
        export: true,
    }));
    world.apply(&ran);
    assert_eq!(world.vars["SHELF_NODE_VERSION"], "22.1.0");

    let ran = world.nvm(&["deactivate"]);
    world.apply(&ran);
    assert!(!world.vars.contains_key("SHELF_NODE_VERSION"));

    let ran = world.nvm(&["use", "20"]);
    world.apply(&ran);
    assert_eq!(world.vars["SHELF_NODE_VERSION"], "20.11.1");
    let ran = world.nvm(&["unload"]);
    world.apply(&ran);
    assert!(!world.vars.contains_key("SHELF_NODE_VERSION"));

    // A new shell that switches to the default version while it loads is
    // told the same thing, in the text `init` prints.
    fs::write(world.shelf.aliases.join("default"), "20\n").unwrap();
    // `nvm unload` took `NVM_DIR` with it.
    world.vars.insert(
        "NVM_DIR".to_string(),
        world.root.join(".nvm").display().to_string(),
    );
    world
        .vars
        .insert("PATH".to_string(), "/usr/bin:/bin".to_string());
    let shell = nvmimic::init::Shell {
        env: world.vars.clone(),
        dialect: Dialect::Bash,
        caller: CallerShell::default(),
        cache_dir: None,
        identity: &SHELFY,
    };
    let load = nvmimic::init::load(&["NVM_DIR", "PATH"], shell, &world.shelf).unwrap();
    assert!(
        load.contains("export SHELF_NODE_VERSION='20.11.1'\n"),
        "{load}"
    );
}

#[test]
fn what_the_shelf_does_not_offer_is_turned_down_when_it_is_run() {
    let mut world = World::new();
    world.with(&["v22.1.0"]);

    for args in [&["install", "-s", "20"][..], &["i", "--lts", "-s"]] {
        let ran = world.nvm(args);
        assert_eq!(ran.outcome.status, 1, "{args:?}");
        assert_eq!(
            ran.err,
            format!("`nvm {} -s` isn't available with this setup.\n", args[0])
        );
        assert_eq!(ran.out, "");
    }
    assert!(world.shelf.asked.borrow().is_empty());

    // What it does offer runs as always, and so does anything after `--`.
    let ran = world.nvm(&["use", "22"]);
    assert_eq!(ran.outcome.status, 0, "{}", ran.err);
    let ran = world.nvm(&["exec", "22", "--", "node", "-s"]);
    assert!(!ran.err.contains("isn't available"), "{}", ran.err);

    // `NVM_SYMLINK_CURRENT` asks for a link in `$NVM_DIR`. The shelf has
    // no such link, so none is made.
    world
        .vars
        .insert("NVM_SYMLINK_CURRENT".to_string(), "true".to_string());
    let ran = world.nvm(&["use", "22"]);
    assert_eq!(ran.outcome.status, 0, "{}", ran.err);
    world.assert_nvm_dir_untouched();

    // `iojs` is not among the names TAB always suggests.
    let ran = world.call("__nvm_complete", &["bash", "2", "nvm", "use", ""]);
    let names: Vec<&str> = ran.out.lines().collect();
    assert!(
        names.contains(&"stable") && !names.contains(&"iojs"),
        "{names:?}"
    );
}

#[test]
fn the_cache_and_the_package_list_are_the_shelf_s_own() {
    let world = World::new();
    world.with(&["v22.1.0"]);
    let nvm_cache = world.root.join(".nvm/.cache");
    fs::create_dir_all(nvm_cache.join("bin")).unwrap();
    fs::write(nvm_cache.join("bin/kept"), "official nvm's download\n").unwrap();
    fs::create_dir_all(world.shelf.cache.join("bin")).unwrap();
    fs::write(world.shelf.cache.join("bin/gone"), "the shelf's download\n").unwrap();

    let ran = world.nvm(&["cache", "dir"]);
    assert_eq!(ran.out, format!("{}\n", world.shelf.cache.display()));

    let mut with_tools = World::new();
    with_tools.shelf = Shelf::at(&world.root);
    with_tools
        .vars
        .insert("PATH".to_string(), "/usr/bin:/bin".to_string());
    with_tools.vars.insert(
        "NVM_DIR".to_string(),
        world.root.join(".nvm").display().to_string(),
    );
    let ran = with_tools.nvm(&["cache", "clear"]);
    assert_eq!(ran.out, "nvm cache cleared.\n", "{}", ran.err);
    assert!(!world.shelf.cache.join("bin/gone").exists());
    assert!(world.shelf.cache.is_dir());
    assert!(nvm_cache.join("bin/kept").is_file());

    // `default-packages` in `$NVM_DIR` belongs to official nvm. The shelf
    // reads its own list.
    fs::write(world.root.join(".nvm/default-packages"), "not-this-one\n").unwrap();
    fs::write(&world.shelf.default_packages, "typescript\n").unwrap();
    let ran = world.call("nvm_get_default_packages", &[]);
    assert_eq!(ran.out, "typescript");

    // The path helpers that scripts call go through the shelf too.
    let listed = format!("{}/bin:/usr/bin", world.path_of("v22.1.0"));
    let ran = world.call("nvm_strip_path", &[&listed, "/bin"]);
    assert_eq!(ran.out, "/usr/bin");
    let ran = world.call(
        "nvm_change_path",
        &["/usr/bin", "/bin", &world.path_of("v22.1.0")],
    );
    assert_eq!(ran.out, format!("{listed}\n"));
}

#[test]
fn a_version_the_shelf_will_not_remove_is_not_called_uninstalled() {
    let world = World::new();
    world.with(&["v20.11.1", "v22.1.0"]);
    fs::write(world.shelf.folder("v20.11.1").join("keep"), "").unwrap();
    fs::write(world.shelf.aliases.join("old"), "v20.11.1\n").unwrap();

    let ran = world.nvm(&["uninstall", "20"]);
    assert_eq!(ran.outcome.status, 1);
    assert_eq!(ran.out, "");
    assert!(
        ran.err
            .ends_with("nvm: could not uninstall v20.11.1: the shelf keeps this one\n"),
        "{}",
        ran.err
    );
    assert!(world.shelf.folder("v20.11.1").join("bin/node").exists());
    assert!(world.shelf.aliases.join("old").exists());
}

#[test]
fn an_install_through_the_shelf_keeps_what_was_asked_for() {
    let mut world = World::new();
    world
        .vars
        .insert("PATH".to_string(), "/usr/bin:/bin".to_string());
    world
        .vars
        .insert("NVM_NODEJS_ORG_MIRROR".to_string(), serve_index());
    // A hook would take the install away from the shelf. The shelf turns
    // hooks down, so this one never runs.
    let hook = world.root.join("hook");
    fs::write(&hook, "#!/bin/sh\ntouch \"$(dirname \"$0\")/hook-ran\"\n").unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    world.vars.insert(
        "NVM_INSTALL_THIRD_PARTY_HOOK".to_string(),
        hook.display().to_string(),
    );

    let ran = world.nvm(&[
        "install",
        "-b",
        "--no-progress",
        "--alias=mine",
        "22",
        "--with-intl=none",
    ]);
    assert_eq!(ran.outcome.status, 0, "{}{}", ran.out, ran.err);
    assert!(!world.root.join("hook-ran").exists());
    assert!(!ran.err.contains("THIRD_PARTY_HOOK"), "{}", ran.err);
    assert_eq!(
        *world.shelf.asked.borrow(),
        [
            "v22.1.0 source:false binary-only:true no-progress:true jobs: configure: --with-intl=none"
        ]
    );
    assert_eq!(
        fs::read_to_string(world.shelf.aliases.join("mine")).unwrap(),
        "22\n"
    );

    let ran = world.nvm(&["install", "-j", "3", "--default", "20"]);
    assert_eq!(ran.outcome.status, 0, "{}{}", ran.out, ran.err);
    assert_eq!(
        world.shelf.asked.borrow()[1],
        "v20.11.1 source:false binary-only:false no-progress:false jobs:3 configure:"
    );
    assert_eq!(
        fs::read_to_string(world.shelf.aliases.join("default")).unwrap(),
        "20\n"
    );
}

// From here on: the second identity. `shelfy` is installed, loaded and
// removed the way nvmimic is, and nothing it leaves carries nvmimic's name.

use nvmimic::dirs::Dirs;
use nvmimic::identity::Identity;
use nvmimic::install::{self, Env, Manifest, Options, Ui};

/// A `Ui` that keeps what it was told and says yes.
#[derive(Default)]
struct Said(Vec<String>);

impl Ui for Said {
    fn say(&mut self, line: &str) {
        self.0.push(line.to_string());
    }

    fn ask(&mut self, _question: &str, _default: bool) -> bool {
        true
    }
}

const SCRIPTS: [&str; 3] = ["nvm.sh", "nvm-exec", "bash_completion"];

/// A throwaway home for an installer run. It is left behind and swept by a
/// later run, because the tests that start programs copied into it must
/// not delete them while macOS may still be scanning them.
fn home(label: &str) -> PathBuf {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    sweep_stale();
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!(
        "nvmimic-shelfy-{label}-{}-{id}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    root.canonicalize_after_create()
}

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
                .starts_with("nvmimic-shelfy-");
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

/// Official nvm's three scripts in `~/.nvm`, each holding its own name.
fn official_nvm(home: &Path) {
    fs::create_dir_all(home.join(".nvm")).unwrap();
    for name in SCRIPTS {
        fs::write(home.join(".nvm").join(name), format!("official {name}\n")).unwrap();
    }
}

/// What the installer sees for a program with this identity in `home`.
fn env_for(identity: &'static Identity, home: &Path) -> Env {
    let at_home = home.display().to_string();
    Env {
        home: home.to_path_buf(),
        nvm_dir: None,
        xdg_config_home: None,
        homebrew_prefix: Some(format!("{at_home}/no-homebrew")),
        cargo_home: None,
        path: None,
        dirs: Dirs::resolve_for(identity, |name| (name == "HOME").then(|| at_home.clone())),
        install_path_given: false,
        identity,
    }
}

fn force() -> Options {
    Options {
        force: true,
        ..Options::default()
    }
}

/// A stand-in for the program file that `self install` copies.
fn program_file(home: &Path) -> PathBuf {
    let program = home.join("downloaded-program");
    fs::write(&program, "a stand-in for the program\n").unwrap();
    program
}

#[test]
fn a_second_program_keeps_its_files_under_its_own_name() {
    let dirs = Dirs::resolve_for(&SHELFY, |name| match name {
        "HOME" => Some("/h".to_string()),
        "SHELFY_STATE_DIR" => Some("/elsewhere/state".to_string()),
        // nvmimic's settings mean nothing to shelfy.
        "NVMIMIC_CACHE_DIR" | "NVMIMIC_INSTALL_PATH" => Some("/not/used".to_string()),
        _ => None,
    })
    .unwrap();
    assert_eq!(dirs.binary, Path::new("/h/.local/bin/shelfy"));
    assert_eq!(dirs.data, Path::new("/h/.local/share/shelfy"));
    assert_eq!(dirs.state, Path::new("/elsewhere/state"));
    assert!(dirs.cache.ends_with("shelfy"), "{:?}", dirs.cache);
}

#[test]
fn a_second_program_installs_and_leaves_under_its_own_name() {
    let home = home("names");
    official_nvm(&home);
    let env = env_for(&SHELFY, &home);
    let mut ui = Said::default();
    install::self_install(&env, &program_file(&home), &force(), &mut ui).unwrap();

    let state = home.join(".local/state/shelfy");
    let manifest = fs::read_to_string(state.join("manifest")).unwrap();
    assert!(
        manifest.starts_with("# What `shelfy self install` changed."),
        "{manifest}"
    );
    assert!(manifest.contains("version 3.1.4\n"), "{manifest}");
    assert!(home.join(".local/bin/shelfy").is_file());
    assert!(!home.join(".local/state/nvmimic").exists());
    assert!(!home.join(".local/bin/nvmimic").exists());

    // The scripts name shelfy wherever nvmimic's name nvmimic, except on
    // the line that says whose code this is.
    let nvm_sh = fs::read_to_string(home.join(".nvm/nvm.sh")).unwrap();
    assert!(nvm_sh.starts_with("# Node Version Manager, as shelfy provides it.\n"));
    assert!(nvm_sh.contains("__SHELFY_PROGRAM="), "{nvm_sh}");
    assert!(nvm_sh.contains("the shelfy program is not at"), "{nvm_sh}");
    let completion = fs::read_to_string(home.join(".nvm/bash_completion")).unwrap();
    assert!(completion.contains("__shelfy_handoff __nvm_complete"));
    assert!(completion.contains("complete -o default -F __nvm shelfy"));
    let bound = install::Bound {
        binary: Path::new("/bin/shelfy"),
        cache: None,
    };
    let init = install::init_text(&SHELFY, &bound, None, None, "");
    assert!(init.contains("__shelfy_handoff() {"));
    assert!(
        init.contains("__SHELFY_CODE='3.1.4'"),
        "the program's own version"
    );
    // The home folder here has `nvmimic` in its name, and a path is never
    // rewritten.
    assert!(nvm_sh.contains(&format!("'{}/.local/bin/shelfy'", home.display())));
    let at_home = home.display().to_string();
    for text in [&nvm_sh, &completion, &init] {
        let text = text.replace(&at_home, "HOME");
        let others: Vec<&str> = text
            .lines()
            .filter(|line| line.to_lowercase().contains("nvmimic"))
            .collect();
        assert!(
            others
                .iter()
                .all(|line| line.contains("nvmimic is MIT licensed")),
            "{others:?}"
        );
    }
    assert!(ui.0.iter().any(|line| line.contains("shelfy")));
    let said = ui.0.join("\n").replace(&at_home, "HOME");
    assert!(!said.contains("nvmimic"), "{said}");

    // shelfy publishes no releases, so there is nothing to update from.
    let source = nvmimic::update::Source::from_process_for(&SHELFY);
    assert!(source.is_none());
    let none = nvmimic::update::Source {
        latest: "http://127.0.0.1:1/none".to_string(),
        allow_unsigned: true,
    };
    assert_eq!(
        nvmimic::update::self_update(&env, &none, &force(), &mut Said::default()),
        Err("there are no shelfy releases for this system yet.".to_string())
    );
    assert_eq!(
        nvmimic::update::self_rollback(&env, &force(), &mut Said::default()),
        Err("there is no earlier shelfy to go back to.".to_string())
    );

    // A script someone edited is kept under shelfy's name, and official
    // nvm's scripts come back.
    fs::write(home.join(".nvm/nvm.sh"), format!("{nvm_sh}# mine\n")).unwrap();
    let mut ui = Said::default();
    install::self_uninstall(&env, &force(), &mut ui).unwrap();
    assert!(home.join(".nvm/nvm.sh.shelfy-saved").is_file());
    for name in SCRIPTS {
        let text = fs::read_to_string(home.join(".nvm").join(name)).unwrap();
        assert_eq!(text, format!("official {name}\n"));
    }
    assert!(!state.exists());
    assert!(ui.0.contains(&"Done. shelfy is gone and nvm is back.".to_string()));
}

#[test]
fn a_manifest_carries_lines_of_the_programs_own() {
    let home = home("manifest");
    let env = env_for(&SHELFY, &home);
    install::self_install(&env, &program_file(&home), &force(), &mut Said::default()).unwrap();
    let dirs = env.dirs.as_ref().unwrap();

    let mut manifest = Manifest::read(dirs).unwrap();
    manifest
        .extra
        .push(("shelf".to_string(), "/a b/shelf".to_string()));
    manifest.write(dirs, &SHELFY).unwrap();
    // Installing again keeps a line the library doesn't know.
    install::self_install(
        &env,
        &home.join(".local/bin/shelfy"),
        &force(),
        &mut Said::default(),
    )
    .unwrap();
    let again = Manifest::read(dirs).unwrap();
    assert_eq!(again.extra("shelf").collect::<Vec<_>>(), ["/a b/shelf"]);
}

/// A stand-in for a shelfy program of one version: it answers `--version`
/// and says yes to `self install`.
#[cfg(target_os = "macos")]
fn shelfy_of(path: &Path, version: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        format!("#!/bin/sh\ncase \"$1\" in --version) echo {version} ;; esac\n"),
    )
    .unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(target_os = "macos")]
#[test]
fn a_second_program_updates_itself_from_its_own_releases() {
    // shelfy as it would be with releases of its own.
    static RELEASED: Identity = Identity {
        name: "shelfy",
        version: "3.1.4",
        completion: &shelf::COMPLETION,
        packages: nvmimic::identity::Packages {
            homebrew: "shelfy",
            cargo: "shelfy-cli",
        },
        release: Some(nvmimic::identity::Release {
            url: "https://example.invalid/shelfy/releases",
            team_id: None,
        }),
    };
    let home = home("update");
    let env = env_for(&RELEASED, &home);
    let program = home.join("downloaded/shelfy");
    shelfy_of(&program, "3.1.4");
    install::self_install(&env, &program, &force(), &mut Said::default()).unwrap();

    // A release is a folder with the archive, its checksum and its version.
    // The archive and the folder inside it carry the program's name.
    let release = home.join("release");
    shelfy_of(&release.join("shelfy/shelfy"), "3.2.0");
    let archive = nvmimic::update::archive_name_for(&RELEASED).unwrap();
    assert_eq!(archive, "shelfy-darwin-universal.tar.gz");
    let packed = std::process::Command::new("tar")
        .arg("-czf")
        .arg(release.join(&archive))
        .arg("-C")
        .arg(&release)
        .arg("shelfy")
        .status()
        .unwrap();
    assert!(packed.success());
    let sum = std::process::Command::new("shasum")
        .args(["-a", "256"])
        .arg(release.join(&archive))
        .output()
        .unwrap();
    let sum = String::from_utf8_lossy(&sum.stdout);
    let sum = sum.split_whitespace().next().unwrap();
    fs::write(release.join("SHA256SUMS"), format!("{sum}  {archive}\n")).unwrap();
    fs::write(release.join("VERSION"), "3.2.0\n").unwrap();
    let source = nvmimic::update::Source {
        latest: format!("file://{}", release.display()),
        allow_unsigned: true,
    };

    let installed = home.join(".local/bin/shelfy");
    let version = || {
        let out = std::process::Command::new(&installed)
            .arg("--version")
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    let mut ui = Said::default();
    nvmimic::update::self_update(&env, &source, &force(), &mut ui).unwrap();
    assert_eq!(version(), "3.2.0");
    assert!(
        ui.0.contains(
            &"Updated shelfy from 3.1.4 to 3.2.0. `shelfy self rollback` goes back to 3.1.4."
                .to_string()
        ),
        "{:?}",
        ui.0
    );
    let state = home.join(".local/state/shelfy");
    assert!(state.join("previous/shelfy-3.1.4").is_file());

    nvmimic::update::self_rollback(&env, &force(), &mut Said::default()).unwrap();
    assert_eq!(version(), "3.1.4");
    assert!(state.join("previous/shelfy-3.2.0").is_file());
    assert!(!state.join("previous/shelfy-3.1.4").exists());
}

#[test]
fn the_programs_own_steps_run_inside_install_and_uninstall() {
    let home = home("steps");
    let shelf = Shelf::at(&home);
    let steps = ShelfSteps(&shelf);
    let env = env_for(&SHELFY, &home);
    let program = program_file(&home);
    let aliases = shelf.aliases.display().to_string();

    // A dry run lists the program's own step with the library's.
    let dry = Options {
        dry_run: true,
        ..Options::default()
    };
    let mut ui = Said::default();
    install::self_install_with(&env, &program, &dry, &mut ui, &steps).unwrap();
    assert_eq!(
        ui.0.last().unwrap(),
        &format!("Would make the shelf's alias folder at {aliases}.")
    );
    assert!(!shelf.aliases.exists());

    let mut ui = Said::default();
    install::self_install_with(&env, &program, &force(), &mut ui, &steps).unwrap();
    assert!(shelf.aliases.is_dir());
    assert!(ui.0.contains(&format!("The shelf's aliases are in {aliases}.")));
    let dirs = env.dirs.as_ref().unwrap();
    let manifest = Manifest::read(dirs).unwrap();
    assert_eq!(
        manifest.extra("shelf-aliases").collect::<Vec<_>>(),
        [&aliases]
    );

    // There was no nvm before shelfy, so an uninstall has no `nvm.sh` to
    // put back. shelfy carries one, and that is what is left.
    let mut ui = Said::default();
    install::self_uninstall_with(&env, &dry, &mut ui, &steps).unwrap();
    let nvm_dir = home.join(".nvm");
    assert!(
        ui.0.contains(&format!(
            "Would write official nvm's nvm.sh into {} from the copy shelfy carries.",
            nvm_dir.display()
        )),
        "{:?}",
        ui.0
    );
    assert!(ui.0.contains(&format!("Would remove {aliases} if it is empty.")));

    let mut ui = Said::default();
    install::self_uninstall_with(&env, &force(), &mut ui, &steps).unwrap();
    assert_eq!(
        fs::read_to_string(nvm_dir.join("nvm.sh")).unwrap(),
        CARRIED_NVM_SH
    );
    assert!(!nvm_dir.join("nvm-exec").exists());
    assert!(!shelf.aliases.exists());
    assert!(!dirs.state.exists());
    assert_eq!(
        ui.0.last().unwrap(),
        "Done. shelfy is gone and nvm is back."
    );
}

#[test]
fn a_carried_copy_is_only_for_when_there_is_nothing_to_put_back() {
    // With an official nvm to go back to, that one comes back.
    let home = home("carried");
    official_nvm(&home);
    let shelf = Shelf::at(&home);
    let steps = ShelfSteps(&shelf);
    let env = env_for(&SHELFY, &home);
    let program = program_file(&home);
    install::self_install_with(&env, &program, &force(), &mut Said::default(), &steps).unwrap();
    install::self_uninstall_with(&env, &force(), &mut Said::default(), &steps).unwrap();
    assert_eq!(
        fs::read_to_string(home.join(".nvm/nvm.sh")).unwrap(),
        "official nvm.sh\n"
    );

    // Without one, an eject has something to hand the folder back to. The
    // library alone refuses, because it carries no copy.
    let home = self::home("carried-eject");
    let shelf = Shelf::at(&home);
    let steps = ShelfSteps(&shelf);
    let env = env_for(&SHELFY, &home);
    let program = program_file(&home);
    install::self_install_with(&env, &program, &force(), &mut Said::default(), &steps).unwrap();
    let refused = install::self_eject(&env, &force(), &mut Said::default());
    assert!(
        refused
            .unwrap_err()
            .starts_with("there is no nvm to go back to.")
    );
    let mut ui = Said::default();
    install::self_eject_with(&env, &force(), &mut ui, &steps).unwrap();
    assert_eq!(
        fs::read_to_string(home.join(".nvm/nvm.sh")).unwrap(),
        CARRIED_NVM_SH
    );
    assert!(home.join(".local/bin/shelfy").is_file());
    assert!(
        shelf.aliases.is_dir(),
        "an eject keeps the program's own files"
    );
    // And shelfy takes over again from the copy it left, like from any nvm.
    install::self_install_with(&env, &program, &force(), &mut Said::default(), &steps).unwrap();
    let nvm_sh = fs::read_to_string(home.join(".nvm/nvm.sh")).unwrap();
    assert_eq!(nvmimic::identity::program_of(&nvm_sh), Some("shelfy"));
}

#[test]
fn a_step_of_the_programs_own_that_fails_leaves_official_nvm_working() {
    struct Fails;
    impl install::Steps for Fails {
        fn install(
            &self,
            _site: &install::Site,
            _manifest: &mut Manifest,
            _ui: &mut dyn Ui,
        ) -> Result<(), String> {
            Err("the shelf fell over".to_string())
        }
    }
    let home = home("failing");
    official_nvm(&home);
    let env = env_for(&SHELFY, &home);
    let stopped = install::self_install_with(
        &env,
        &program_file(&home),
        &force(),
        &mut Said::default(),
        &Fails,
    );
    assert_eq!(stopped, Err("the shelf fell over".to_string()));
    for name in SCRIPTS {
        let text = fs::read_to_string(home.join(".nvm").join(name)).unwrap();
        assert_eq!(text, format!("official {name}\n"));
    }
    assert!(!home.join(".local/bin/shelfy").exists());
    // What the stopped install left is cleared by an uninstall.
    install::self_uninstall(&env, &force(), &mut Said::default()).unwrap();
    assert!(!home.join(".local/state/shelfy/manifest").exists());
    assert_eq!(
        fs::read_to_string(home.join(".nvm/nvm.sh")).unwrap(),
        "official nvm.sh\n"
    );
}

#[test]
fn two_programs_never_share_an_nvm_folder() {
    for (first, second) in [(&SHELFY, &Identity::NVMIMIC), (&Identity::NVMIMIC, &SHELFY)] {
        let home = home("shared");
        official_nvm(&home);
        let program = program_file(&home);
        install::self_install(
            &env_for(first, &home),
            &program,
            &force(),
            &mut Said::default(),
        )
        .unwrap();
        let before = fs::read(home.join(".nvm/nvm.sh")).unwrap();

        let mut ui = Said::default();
        let refused = install::self_install(&env_for(second, &home), &program, &force(), &mut ui);
        let (one, other) = (first.name, second.name);
        assert_eq!(
            refused,
            Err(format!(
                "{one}'s scripts are in {}, and {other} and {one} can't share an nvm folder. Run \
                 `{one} self eject` or `{one} self uninstall` first. Nothing was changed.",
                home.join(".nvm").display()
            ))
        );
        assert_eq!(fs::read(home.join(".nvm/nvm.sh")).unwrap(), before);
        assert!(!home.join(".local/state").join(other).exists());
        assert!(!home.join(".local/bin").join(other).exists());
        assert!(ui.0.is_empty(), "{:?}", ui.0);
        // A dry run is refused the same way.
        let dry = Options {
            dry_run: true,
            ..Options::default()
        };
        assert!(
            install::self_install(
                &env_for(second, &home),
                &program,
                &dry,
                &mut Said::default()
            )
            .is_err()
        );

        // Once the first program is out, the second one takes over from
        // official nvm like any other install.
        install::self_eject(&env_for(first, &home), &force(), &mut Said::default()).unwrap();
        install::self_install(
            &env_for(second, &home),
            &program,
            &force(),
            &mut Said::default(),
        )
        .unwrap();
        let nvm_sh = fs::read_to_string(home.join(".nvm/nvm.sh")).unwrap();
        assert_eq!(nvmimic::identity::program_of(&nvm_sh), Some(other));
        // And now the first is the one that is refused.
        assert!(
            install::self_install(
                &env_for(first, &home),
                &program,
                &force(),
                &mut Said::default()
            )
            .is_err()
        );
    }
}

#[test]
fn unload_and_completion_go_by_the_programs_name() {
    let world = World::new();
    let ran = world.nvm(&["unload"]);
    let unset: Vec<&String> = ran
        .outcome
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::UnsetFunctions(names) => Some(names),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(unset.iter().any(|name| *name == "__shelfy_handoff"));
    assert!(unset.iter().all(|name| !name.contains("nvmimic")));

    let ran = world.call("__nvm_complete", &["bash", "1", "shelfy", ""]);
    assert_eq!(ran.out, "init\nself\n");
    // nvmimic's own command means nothing to shelfy's shells.
    let ran = world.call("__nvm_complete", &["bash", "1", "nvmimic", ""]);
    assert!(ran.out.contains("install"), "nvm's own table: {}", ran.out);
    assert!(!ran.out.contains("self"), "{}", ran.out);
}

/// The `shelfy` program: the example in `examples/shelfy.rs`. `cargo test`
/// builds it beside the tests. A run of this test alone builds it here.
fn shelfy_program() -> PathBuf {
    let tests = std::env::current_exe().unwrap();
    let program = tests
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .join("examples/shelfy");
    if !program.is_file() {
        let mut build = std::process::Command::new(env!("CARGO"));
        build.args(["build", "--quiet", "-p", "nvmimic", "--example", "shelfy"]);
        if !cfg!(debug_assertions) {
            build.arg("--release");
        }
        assert!(build.status().unwrap().success(), "building shelfy");
    }
    program
}

fn shell_programs() -> Vec<(&'static str, &'static [&'static str])> {
    let all: [(&str, &[&str]); 4] = [
        ("/bin/bash", &[]),
        ("/bin/zsh", &["-f"]),
        ("/bin/sh", &[]),
        ("/bin/dash", &[]),
    ];
    all.into_iter()
        .filter(|(program, _)| Path::new(program).is_file())
        .collect()
}

#[test]
fn real_shells_run_nvm_through_the_second_program() {
    let home = home("shell");
    let shelfy = shelfy_program();
    let run = |args: &[&str]| {
        std::process::Command::new(&shelfy)
            .args(args)
            .env_clear()
            .env("HOME", &home)
            .env("PATH", "/usr/bin:/bin")
            .env("HOMEBREW_PREFIX", home.join("no-homebrew"))
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap()
    };
    let installed = run(&["self", "install", "--force"]);
    assert!(
        installed.status.success(),
        "{}",
        String::from_utf8_lossy(&installed.stderr)
    );
    let shelf = Shelf::at(&home);
    for version in ["v20.1.0", "v22.1.0"] {
        fake_node(&shelf.folder(version), version);
    }
    fs::create_dir_all(&shelf.aliases).unwrap();
    fs::write(shelf.aliases.join("default"), "20\n").unwrap();

    let script = r#"
. "$NVM_DIR/nvm.sh" || exit 90
echo "loaded:${SHELFY_LOADED-} default:$(nvm current) shelf:${SHELF_NODE_VERSION-}"
nvm use 22.1.0 || exit 91
echo "node:$(command -v node) shelf:${SHELF_NODE_VERSION-}"
echo "aliases:$(nvm alias default 22 >/dev/null && cat "$HOME/shelf-aliases/default")"
case "$(command -v __shelfy_handoff)" in __shelfy_handoff) echo own:yes ;; esac
command -v __nvmimic_handoff >/dev/null 2>&1 && echo "nvmimic's function is here"
nvm unload
command -v __shelfy_handoff >/dev/null 2>&1 && echo "still loaded"
command -v nvm >/dev/null 2>&1 && echo "nvm still loaded"
echo done
"#;
    for (shell, flags) in shell_programs() {
        fs::write(shelf.aliases.join("default"), "20\n").unwrap();
        let out = std::process::Command::new(shell)
            .args(flags)
            .args(["-c", script])
            .env_clear()
            .env("HOME", &home)
            .env("NVM_DIR", home.join(".nvm"))
            .env("PATH", "/usr/bin:/bin")
            .output()
            .unwrap();
        let said = String::from_utf8_lossy(&out.stdout);
        let errors = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            said,
            format!(
                "loaded:1 default:v20.1.0 shelf:20.1.0\nNow using node v22.1.0 (npm v10.7.0)\n\
                 node:{}/shelf/22.1.0/bin/node shelf:22.1.0\naliases:22\nown:yes\ndone\n",
                home.display()
            ),
            "{shell}: {errors}"
        );
        assert_eq!(errors, "", "{shell}");
    }
    // Nothing of nvmimic's own appeared, and nothing went into `$NVM_DIR`
    // but the three scripts.
    assert!(!home.join(".local/state/nvmimic").exists());
    let mut in_nvm_dir: Vec<String> = fs::read_dir(home.join(".nvm"))
        .unwrap()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    in_nvm_dir.sort();
    assert_eq!(in_nvm_dir, ["bash_completion", "nvm-exec", "nvm.sh"]);
}
