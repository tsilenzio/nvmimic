//! Where nvmimic keeps its own files.
//!
//! None of them go into `$NVM_DIR`. That folder holds what nvm keeps there
//! and nothing else, so a folder made by real nvm and one used by nvmimic
//! look the same. nvmimic's own files follow the layout many command line
//! tools use, with one variable to move each part:
//!
//! | What | Variable | Otherwise |
//! |---|---|---|
//! | the program | `NVMIMIC_INSTALL_PATH` | `~/.local/bin/nvmimic` |
//! | files that belong to the installation | `NVMIMIC_DATA_DIR` | `$XDG_DATA_HOME/nvmimic`, `~/.local/share/nvmimic` |
//! | the record of what an install changed | `NVMIMIC_STATE_DIR` | `$XDG_STATE_HOME/nvmimic`, `~/.local/state/nvmimic` |
//! | remembered answers, safe to delete | `NVMIMIC_CACHE_DIR` | `$XDG_CACHE_HOME/nvmimic`, `~/Library/Caches/nvmimic` on macOS, `~/.cache/nvmimic` elsewhere |
//!
//! Another program built on this library has the same layout under its own
//! name, see [`Dirs::resolve_for`].
//!
//! One folder belongs to no single program: the one nvmimic and its sister
//! tools share, see [`crate::handover`]. Every one of them has to find it
//! in the same place, so no program's own setting moves it. Its standard
//! place is `NVMIMIC_SHARED_DIR`, otherwise `$XDG_STATE_HOME/nvmimic-shared`,
//! otherwise `~/.local/state/nvmimic-shared`. Each program keeps a link
//! named `shared` in its own state folder that points at it, and finds the
//! folder through that link ([`Dirs::shared_folder`]).

use crate::identity::Identity;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dirs {
    /// The nvmimic program itself.
    pub binary: PathBuf,
    /// Files that belong to the installation. Nothing is kept here now.
    pub data: PathBuf,
    /// The manifest and earlier programs.
    pub state: PathBuf,
    /// Remembered answers.
    pub cache: PathBuf,
    /// The standard place of the folder shared with sister tools, the
    /// same under every identity. The folder in use is
    /// [`Dirs::shared_folder`].
    pub shared: PathBuf,
}

/// The name of the shared folder and the variable that moves it.
const SHARED: &str = "nvmimic-shared";
const SHARED_SETTING: &str = "NVMIMIC_SHARED_DIR";

impl Dirs {
    /// Works nvmimic's locations out from the environment. `var` returns
    /// the value of a variable. `None` when a location has no variable set
    /// and there is no home folder to fall back to.
    pub fn resolve(var: impl Fn(&str) -> Option<String>) -> Option<Dirs> {
        Dirs::resolve_for(&Identity::NVMIMIC, var)
    }

    /// [`Dirs::resolve`] for another program built on this library. Its
    /// name takes nvmimic's place in every folder and variable name.
    pub fn resolve_for(identity: &Identity, var: impl Fn(&str) -> Option<String>) -> Option<Dirs> {
        let name = identity.name;
        // A relative path means something else in every folder, so it
        // counts as not set.
        let absolute = |name: &str| {
            var(name)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
        };
        let home = absolute("HOME");
        let pick = |own: &str, xdg: &str, fallback: &str| {
            absolute(&identity.setting(own))
                .or_else(|| absolute(xdg).map(|base| base.join(name)))
                .or_else(|| home.as_ref().map(|home| home.join(fallback).join(name)))
        };
        let cache = if cfg!(target_os = "macos") {
            "Library/Caches"
        } else {
            ".cache"
        };
        Some(Dirs {
            binary: absolute(&identity.setting("INSTALL_PATH"))
                .or_else(|| home.as_ref().map(|home| home.join(".local/bin").join(name)))?,
            data: pick("DATA_DIR", "XDG_DATA_HOME", ".local/share")?,
            state: pick("STATE_DIR", "XDG_STATE_HOME", ".local/state")?,
            cache: pick("CACHE_DIR", "XDG_CACHE_HOME", cache)?,
            shared: absolute(SHARED_SETTING)
                .or_else(|| absolute("XDG_STATE_HOME").map(|base| base.join(SHARED)))
                .or_else(|| {
                    home.as_ref()
                        .map(|home| home.join(".local/state").join(SHARED))
                })?,
        })
    }

    /// nvmimic's locations for this process's environment.
    pub fn from_process() -> Option<Dirs> {
        Dirs::from_process_for(&Identity::NVMIMIC)
    }

    /// [`Dirs::from_process`] for another program built on this library.
    pub fn from_process_for(identity: &Identity) -> Option<Dirs> {
        Dirs::resolve_for(identity, |name| {
            std::env::var(name).ok().filter(|value| !value.is_empty())
        })
    }

    pub fn manifest(&self) -> PathBuf {
        self.state.join("manifest")
    }

    /// The link in the program's own state folder that points at the
    /// shared folder.
    pub fn shared_link(&self) -> PathBuf {
        self.state.join("shared")
    }

    /// The folder shared with sister tools: where the program's link
    /// points, so the folder can be moved by moving it and pointing each
    /// program's link at the new place. With the link missing, or pointing
    /// at nothing, it is the standard place.
    pub fn shared_folder(&self) -> PathBuf {
        let link = self.shared_link();
        match fs::read_link(&link) {
            Ok(target) if fs::metadata(&link).is_ok_and(|meta| meta.is_dir()) => {
                self.state.join(target)
            }
            _ => self.shared.clone(),
        }
    }

    /// The record shared with sister tools.
    pub fn record(&self) -> PathBuf {
        self.shared_folder().join("record")
    }

    /// The note in the shared folder that says what the folder is.
    pub fn shared_readme(&self) -> PathBuf {
        self.shared_folder().join("README")
    }

    /// Where official nvm's own scripts wait while a program's are in
    /// `$NVM_DIR`.
    pub fn backup(&self) -> PathBuf {
        self.shared_folder().join("backup")
    }

    /// Where nvmimic 0.1.0 kept official nvm's scripts, before the shared
    /// record existed.
    pub(crate) fn backup_before_sharing(&self) -> PathBuf {
        self.state.join("backup")
    }
}

/// The folders under the cache folder, one per kind of remembered answer.
pub const CACHE_KINDS: [&str; 2] = ["node-version", "npm-version"];

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn dirs(vars: &[(&str, &str)]) -> Option<Dirs> {
        Dirs::resolve(|name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_string())
        })
    }

    #[test]
    fn defaults_sit_under_home() {
        let dirs = dirs(&[("HOME", "/h")]).unwrap();
        assert_eq!(dirs.binary, Path::new("/h/.local/bin/nvmimic"));
        assert_eq!(dirs.data, Path::new("/h/.local/share/nvmimic"));
        assert_eq!(dirs.state, Path::new("/h/.local/state/nvmimic"));
        assert!(dirs.cache.starts_with("/h"));
        assert_eq!(dirs.shared, Path::new("/h/.local/state/nvmimic-shared"));
    }

    #[test]
    fn own_variables_win_over_xdg() {
        let dirs = dirs(&[
            ("HOME", "/h"),
            ("XDG_DATA_HOME", "/x/data"),
            ("XDG_STATE_HOME", "/x/state"),
            ("NVMIMIC_STATE_DIR", "/own/state"),
            ("NVMIMIC_INSTALL_PATH", "/opt/tools/nvmimic"),
        ])
        .unwrap();
        assert_eq!(dirs.binary, Path::new("/opt/tools/nvmimic"));
        assert_eq!(dirs.data, Path::new("/x/data/nvmimic"));
        assert_eq!(dirs.state, Path::new("/own/state"));
        // The shared record goes by no program's variable.
        assert_eq!(dirs.shared, Path::new("/x/state/nvmimic-shared"));
    }

    #[test]
    fn every_program_finds_the_shared_record_in_the_same_place() {
        let other = Identity {
            name: "shelfy",
            ..Identity::NVMIMIC
        };
        for vars in [
            vec![("HOME", "/h")],
            vec![("HOME", "/h"), ("XDG_STATE_HOME", "/x")],
            vec![("HOME", "/h"), ("NVMIMIC_SHARED_DIR", "/shared")],
        ] {
            let var = |name: &str| {
                vars.iter()
                    .find(|(key, _)| *key == name)
                    .map(|(_, value)| value.to_string())
            };
            let ours = Dirs::resolve(var).unwrap();
            let theirs = Dirs::resolve_for(&other, var).unwrap();
            assert_eq!(ours.shared, theirs.shared);
            assert_ne!(ours.state, theirs.state);
        }
    }

    #[test]
    fn a_relative_path_counts_as_not_set() {
        let dirs = dirs(&[("HOME", "/h"), ("XDG_DATA_HOME", "data")]).unwrap();
        assert_eq!(dirs.data, Path::new("/h/.local/share/nvmimic"));
    }

    #[test]
    fn nothing_without_a_home_or_variables() {
        assert_eq!(dirs(&[]), None);
    }
}
