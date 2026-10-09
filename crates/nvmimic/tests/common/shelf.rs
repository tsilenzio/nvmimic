//! `Shelf`, a place to keep Node versions that looks nothing like nvm's,
//! and `shelfy`, the name of the program built on it. Shared by the tests in
//! `other_store.rs` and the example program in `examples/shelfy.rs`.

use nvmimic::backend::{self, Backend, Flavor, Flavors, InstallRequest, Installed};
use nvmimic::complete::{Command, Needs, command, flag};
use nvmimic::context::Nvm;
use nvmimic::identity::{Identity, Packages};
use nvmimic::install::{Leaving, Manifest, Site, Steps, Ui};
use nvmimic::nvm::paths;
use nvmimic::shell::Vars;
use std::cell::RefCell;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// What TAB suggests after `shelfy`.
pub static COMPLETION: Command = Command {
    name: "shelfy",
    about: "nvm on a shelf",
    needs: Needs::Nothing,
    flags: &[flag("--version", "print shelfy's version")],
    takes: &[],
    commands: &[
        command(
            "init",
            "print the shell code that loads shelfy",
            Needs::Nothing,
            &[],
            &[],
        ),
        command(
            "self",
            "set shelfy up or remove it",
            Needs::Nothing,
            &[],
            &[],
        ),
    ],
};

/// The second identity. Everything nvmimic calls `nvmimic` is `shelfy`
/// here: `~/.local/state/shelfy`, `SHELFY_STATE_DIR`, `__shelfy_handoff`.
pub static SHELFY: Identity = Identity {
    name: "shelfy",
    version: "3.1.4",
    completion: &COMPLETION,
    packages: Packages {
        homebrew: "shelfy",
        cargo: "shelfy-cli",
    },
    release: None,
};

pub struct Shelf {
    pub store: PathBuf,
    pub aliases: PathBuf,
    /// Downloads and install locks, kept away from `$NVM_DIR/.cache`.
    pub cache: PathBuf,
    /// The shelf's own list of packages for every new version.
    pub default_packages: PathBuf,
    /// What each install was asked for, for the tests to read.
    pub asked: RefCell<Vec<String>>,
}

impl Shelf {
    /// The shelf under `root`: versions in `shelf/`, aliases beside it.
    pub fn at(root: &Path) -> Shelf {
        Shelf {
            store: root.join("shelf"),
            aliases: root.join("shelf-aliases"),
            cache: root.join("shelf-cache"),
            default_packages: root.join("shelf-packages"),
            asked: RefCell::new(Vec::new()),
        }
    }

    pub fn folder(&self, version: &str) -> PathBuf {
        self.store.join(version.trim_start_matches('v'))
    }

    pub fn store(&self) -> String {
        self.store.display().to_string()
    }
}

impl Backend for Shelf {
    // The shelf installs versions itself and never builds one. It keeps
    // Node only, has no `current` link, and lets no hook install for it.
    fn offers(&self, needs: Needs) -> bool {
        !matches!(
            needs,
            Needs::Source | Needs::Iojs | Needs::SymlinkCurrent | Needs::InstallHook
        )
    }

    // The shelf keeps a variable of its own in step with the active
    // version, the way a tool with its own idea of "the current Node" does.
    fn switched(&self, vars: &mut Vars, version: Option<&str>) {
        match version {
            Some(version) => vars.export("SHELF_NODE_VERSION", version.trim_start_matches('v')),
            None => vars.unset("SHELF_NODE_VERSION"),
        }
    }

    fn cache_dir(&self, _vars: &Vars) -> String {
        self.cache.display().to_string()
    }

    fn default_packages_file(&self, _vars: &Vars) -> String {
        self.default_packages.display().to_string()
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

    // A version with a `keep` file in it stays on the shelf.
    fn remove(&self, vars: &Vars, version: &str) -> std::io::Result<()> {
        if self.folder(version).join("keep").exists() {
            return Err(std::io::Error::other("the shelf keeps this one"));
        }
        backend::remove_tree(&self.version_path(vars, version))
    }

    fn install(&self, nvm: &mut Nvm, request: &InstallRequest) -> Option<i32> {
        self.asked.borrow_mut().push(format!(
            "{} source:{} binary-only:{} no-progress:{} jobs:{} configure:{}",
            request.version,
            request.from_source,
            request.binary_only,
            request.no_progress,
            request.make_jobs,
            request.configure_args
        ));
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
pub fn fake_node(folder: &Path, version: &str) {
    let bin = folder.join("bin");
    fs::create_dir_all(&bin).unwrap();
    for (name, answer) in [("node", version), ("npm", "10.7.0")] {
        let program = bin.join(name);
        fs::write(&program, format!("#!/bin/sh\necho {answer}\n")).unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

/// A stand-in for the copy of official nvm a program can carry, for the
/// times an uninstall has no `nvm.sh` to put back.
pub const CARRIED_NVM_SH: &str =
    "# official nvm, as shelfy carries it\nnvm() { echo 'official nvm'; }\n";

/// What shelfy adds to `self install`, `self eject` and `self uninstall`:
/// it makes its alias folder, writes the folder's place into the manifest,
/// and takes the folder away again if it is still empty.
pub struct ShelfSteps<'a>(pub &'a Shelf);

impl Steps for ShelfSteps<'_> {
    fn install_plan(&self, _site: &Site) -> Vec<String> {
        vec![format!(
            "make the shelf's alias folder at {}",
            self.0.aliases.display()
        )]
    }

    fn install(
        &self,
        _site: &Site,
        manifest: &mut Manifest,
        ui: &mut dyn Ui,
    ) -> Result<(), String> {
        fs::create_dir_all(&self.0.aliases).map_err(|error| error.to_string())?;
        manifest.extra.retain(|(key, _)| key != "shelf-aliases");
        manifest.extra.push((
            "shelf-aliases".to_string(),
            self.0.aliases.display().to_string(),
        ));
        ui.say(&format!(
            "The shelf's aliases are in {}.",
            self.0.aliases.display()
        ));
        Ok(())
    }

    fn leave_plan(&self, _site: &Site, how: Leaving, manifest: &Manifest) -> Vec<String> {
        match (how, manifest.extra("shelf-aliases").next()) {
            (Leaving::Eject, _) | (_, None) => Vec::new(),
            (_, Some(folder)) => vec![format!("remove {folder} if it is empty")],
        }
    }

    fn leave(
        &self,
        _site: &Site,
        how: Leaving,
        manifest: &mut Manifest,
        _ui: &mut dyn Ui,
    ) -> Result<(), String> {
        if how != Leaving::Eject
            && let Some(folder) = manifest.extra("shelf-aliases").next()
        {
            let _ = fs::remove_dir(folder);
        }
        Ok(())
    }

    fn official_script(&self, _site: &Site, name: &str) -> Option<String> {
        (name == "nvm.sh").then(|| CARRIED_NVM_SH.to_string())
    }
}
