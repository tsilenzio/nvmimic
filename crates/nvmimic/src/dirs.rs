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

use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dirs {
    /// The nvmimic program itself.
    pub binary: PathBuf,
    /// The two files of shell functions `nvm.sh` reads.
    pub data: PathBuf,
    /// The manifest, the backups of nvm's scripts and earlier programs.
    pub state: PathBuf,
    /// Remembered answers.
    pub cache: PathBuf,
}

impl Dirs {
    /// Works the locations out from the environment. `var` returns the value
    /// of a variable. `None` when a location has no variable set and there
    /// is no home folder to fall back to.
    pub fn resolve(var: impl Fn(&str) -> Option<String>) -> Option<Dirs> {
        // A relative path means something else in every folder, so it
        // counts as not set.
        let absolute = |name: &str| {
            var(name)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
        };
        let home = absolute("HOME");
        let pick = |own: &str, xdg: &str, fallback: &str| {
            absolute(own)
                .or_else(|| absolute(xdg).map(|base| base.join("nvmimic")))
                .or_else(|| home.as_ref().map(|home| home.join(fallback)))
        };
        let cache = if cfg!(target_os = "macos") {
            "Library/Caches/nvmimic"
        } else {
            ".cache/nvmimic"
        };
        Some(Dirs {
            binary: absolute("NVMIMIC_INSTALL_PATH")
                .or_else(|| home.as_ref().map(|home| home.join(".local/bin/nvmimic")))?,
            data: pick("NVMIMIC_DATA_DIR", "XDG_DATA_HOME", ".local/share/nvmimic")?,
            state: pick(
                "NVMIMIC_STATE_DIR",
                "XDG_STATE_HOME",
                ".local/state/nvmimic",
            )?,
            cache: pick("NVMIMIC_CACHE_DIR", "XDG_CACHE_HOME", cache)?,
        })
    }

    /// The locations for this process's environment.
    pub fn from_process() -> Option<Dirs> {
        Dirs::resolve(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
    }

    pub fn manifest(&self) -> PathBuf {
        self.state.join("manifest")
    }

    /// Where nvm's own scripts wait while nvmimic's are in `$NVM_DIR`.
    pub fn backup(&self) -> PathBuf {
        self.state.join("backup")
    }

    pub fn nvm_functions(&self) -> PathBuf {
        self.data.join("nvm-functions.sh")
    }

    pub fn nvmimic_functions(&self) -> PathBuf {
        self.data.join("nvmimic-functions.sh")
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
