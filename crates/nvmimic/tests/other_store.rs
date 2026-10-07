//! A second backend, to show that the library does not need nvm's own
//! folder layout. `Shelf` keeps every version in one folder outside
//! `$NVM_DIR`, names the folders without the leading `v`, keeps its aliases
//! in a folder of its own, and installs versions itself. Each test runs
//! nvm's commands through [`nvmimic::run`], the way another program would.

use nvmimic::backend::{self, Backend, Flavor, Flavors, InstallRequest, Installed};
use nvmimic::context::{CallerShell, Nvm};
use nvmimic::nvm::paths;
use nvmimic::shell::{Dialect, Effect, Vars};
use nvmimic::{Caller, Outcome};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

struct Shelf {
    store: PathBuf,
    aliases: PathBuf,
}

impl Shelf {
    fn folder(&self, version: &str) -> PathBuf {
        self.store.join(version.trim_start_matches('v'))
    }

    fn store(&self) -> String {
        self.store.display().to_string()
    }
}

impl Backend for Shelf {
    // The shelf installs versions itself and never builds one.
    fn offers(&self, needs: nvmimic::complete::Needs) -> bool {
        needs != nvmimic::complete::Needs::Source
    }

    fn root(&self, vars: &Vars) -> String {
        vars.get("NVM_DIR").to_string()
    }

    fn installed(&self, _vars: &Vars, flavors: Flavors) -> Vec<Installed> {
        if flavors == Flavors::Iojs {
            return Vec::new();
        }
        backend::dir_entries(&self.store())
            .into_iter()
            .filter(|entry| entry.is_dir)
            .map(|entry| Installed {
                flavor: Flavor::Node,
                name: format!("v{}", entry.name),
            })
            .collect()
    }

    fn version_path(&self, _vars: &Vars, version: &str) -> String {
        self.folder(version).display().to_string()
    }

    fn owner_of(&self, _vars: &Vars, path: &str) -> Option<Flavor> {
        (paths::tree_contains_path(&self.store(), path) == Some(true)).then_some(Flavor::Node)
    }

    fn remove(&self, vars: &Vars, version: &str) -> std::io::Result<()> {
        backend::remove_tree(&self.version_path(vars, version))
    }

    fn install(&self, nvm: &mut Nvm, request: &InstallRequest) -> Option<i32> {
        nvm.io
            .echo(&format!("shelf: putting {} in place", request.version));
        fake_node(&self.folder(request.version), request.version);
        Some(0)
    }

    fn strip_path(&self, _vars: &Vars, list: &str, suffix: &str) -> String {
        paths::strip_path(list, &self.store(), suffix)
    }

    fn change_path(&self, _vars: &Vars, list: &str, suffix: &str, version_path: &str) -> String {
        paths::change_path(list, suffix, version_path, &self.store())
    }

    fn alias_dir(&self, _vars: &Vars) -> String {
        self.aliases.display().to_string()
    }
}

/// A stand-in for an installed Node: a `node` and an `npm` that only know
/// their versions. Without an `npm`, `nvm install` goes and fetches one.
fn fake_node(folder: &Path, version: &str) {
    let bin = folder.join("bin");
    fs::create_dir_all(&bin).unwrap();
    for (name, answer) in [("node", version), ("npm", "10.7.0")] {
        let program = bin.join(name);
        fs::write(&program, format!("#!/bin/sh\necho {answer}\n")).unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

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
        let shelf = Shelf {
            store: root.join("shelf"),
            aliases: root.join("shelf-aliases"),
        };
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
        v22.1.0\t2024-05-02\tosx-arm64-tar,linux-x64,linux-arm64\t10.7.0\t12.4\t1.48.0\t1.3.0\t3.0.13\t127\t-\t-\n";
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
