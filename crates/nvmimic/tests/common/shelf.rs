//! `Shelf`, a place to keep Node versions that looks nothing like nvm's,
//! and `shelfy`, the name of the program built on it. Shared by the tests in
//! `other_store.rs` and the example program in `examples/shelfy.rs`.

use nvmimic::backend::{self, Backend, Flavor, Flavors, InstallRequest, Installed};
use nvmimic::complete::{Command, Needs, command, flag};
use nvmimic::context::Nvm;
use nvmimic::identity::{Identity, Packages};
use nvmimic::nvm::paths;
use nvmimic::shell::Vars;
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
}

impl Shelf {
    /// The shelf under `root`: versions in `shelf/`, aliases beside it.
    pub fn at(root: &Path) -> Shelf {
        Shelf {
            store: root.join("shelf"),
            aliases: root.join("shelf-aliases"),
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
    // The shelf installs versions itself and never builds one.
    fn offers(&self, needs: Needs) -> bool {
        needs != Needs::Source
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
pub fn fake_node(folder: &Path, version: &str) {
    let bin = folder.join("bin");
    fs::create_dir_all(&bin).unwrap();
    for (name, answer) in [("node", version), ("npm", "10.7.0")] {
        let program = bin.join(name);
        fs::write(&program, format!("#!/bin/sh\necho {answer}\n")).unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
    }
}
