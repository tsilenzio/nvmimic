//! Where Node versions and aliases live.
//!
//! Commands, messages and version resolution go through [`Backend`], so a
//! project that keeps Node installs elsewhere can reuse them. [`NvmDirBackend`]
//! is nvm's own `$NVM_DIR` layout and the one parity is measured against.

use crate::shell::Vars;
use std::fs;
use std::path::Path;

/// The three places nvm keeps installs, as `nvm_version_dir` names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirKind {
    /// `versions/node`, for Node 0.12 and later.
    New,
    /// `versions/io.js`.
    Iojs,
    /// `$NVM_DIR` itself, where Node before 0.12 was installed.
    Old,
}

/// What reading an alias file found.
#[derive(Debug, PartialEq, Eq)]
pub enum AliasFile {
    Missing,
    /// The file exists but cannot be read, at this path.
    Unreadable(String),
    /// The raw file contents.
    Contents(String),
}

pub trait Backend {
    /// `$NVM_DIR`, the root every nvm path hangs from.
    fn root(&self, vars: &Vars) -> String;

    /// nvm: nvm_version_dir
    fn version_dir(&self, vars: &Vars, kind: DirKind) -> String {
        let root = self.root(vars);
        match kind {
            DirKind::New => format!("{root}/versions/node"),
            DirKind::Iojs => format!("{root}/versions/io.js"),
            DirKind::Old => root,
        }
    }

    /// The entries of an install directory, as `"${dir}"/*` expands: names
    /// that do not start with a dot, sorted, with real directories marked.
    fn entries(&self, dir: &str) -> Vec<Entry>;

    /// Whether a directory exists and has any entry at all, dot files
    /// included, as `ls -1qA dir | grep -q .` checks.
    fn has_entries(&self, dir: &str) -> bool;

    /// `[ -x "${path}" ]`.
    fn is_executable(&self, path: &str) -> bool;

    /// nvm: nvm_alias_path
    fn alias_dir(&self, vars: &Vars) -> String {
        format!("{}/alias", self.version_dir(vars, DirKind::Old))
    }

    fn read_alias(&self, vars: &Vars, name: &str) -> AliasFile;

    /// Writes an alias file. nvm writes the target followed by a newline.
    fn write_alias(&self, vars: &Vars, name: &str, target: &str) -> std::io::Result<()>;

    fn remove_alias(&self, vars: &Vars, name: &str);

    /// Whether `alias/<name>` is a regular file.
    fn alias_exists(&self, vars: &Vars, name: &str) -> bool;

    /// What `"${alias_dir}/${prefix}"*` expands to, in glob order. When
    /// nothing matches, the shell leaves the pattern itself.
    fn alias_glob(&self, vars: &Vars, prefix: &str) -> Vec<String>;

    /// `mkdir -p` of a directory nvm creates as a side effect.
    fn make_dirs(&self, path: &str);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
}

/// nvm's own layout under `$NVM_DIR`.
pub struct NvmDirBackend;

impl Backend for NvmDirBackend {
    fn root(&self, vars: &Vars) -> String {
        vars.get("NVM_DIR").to_string()
    }

    fn entries(&self, dir: &str) -> Vec<Entry> {
        let Ok(read) = fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut entries: Vec<Entry> = read
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name().into_string().ok()?;
                if name.starts_with('.') {
                    return None;
                }
                // `find -type d` does not follow symlinks.
                let is_dir = entry.file_type().is_ok_and(|kind| kind.is_dir());
                Some(Entry { name, is_dir })
            })
            .collect();
        entries.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));
        entries
    }

    fn has_entries(&self, dir: &str) -> bool {
        fs::read_dir(dir).is_ok_and(|mut read| read.next().is_some())
    }

    fn is_executable(&self, path: &str) -> bool {
        is_executable(Path::new(path))
    }

    fn read_alias(&self, vars: &Vars, name: &str) -> AliasFile {
        let path = format!("{}/{name}", self.alias_dir(vars));
        if !Path::new(&path).is_file() {
            return AliasFile::Missing;
        }
        match fs::read(&path) {
            Ok(bytes) => AliasFile::Contents(String::from_utf8_lossy(&bytes).into_owned()),
            Err(_) => AliasFile::Unreadable(path),
        }
    }

    fn write_alias(&self, vars: &Vars, name: &str, target: &str) -> std::io::Result<()> {
        fs::write(
            format!("{}/{name}", self.alias_dir(vars)),
            format!("{target}\n"),
        )
    }

    fn remove_alias(&self, vars: &Vars, name: &str) {
        let _ = fs::remove_file(format!("{}/{name}", self.alias_dir(vars)));
    }

    fn alias_exists(&self, vars: &Vars, name: &str) -> bool {
        Path::new(&format!("{}/{name}", self.alias_dir(vars))).is_file()
    }

    fn alias_glob(&self, vars: &Vars, prefix: &str) -> Vec<String> {
        let dir = self.alias_dir(vars);
        glob_prefix(&dir, prefix)
    }

    fn make_dirs(&self, path: &str) {
        let _ = fs::create_dir_all(path);
    }
}

/// Expands `"${dir}/${prefix}"*` the way a POSIX shell does. The prefix may
/// itself contain a `/`, as in `lts/`.
pub fn glob_prefix(dir: &str, prefix: &str) -> Vec<String> {
    let (sub, name_prefix) = match prefix.rfind('/') {
        Some(at) => (format!("{dir}/{}", &prefix[..at]), &prefix[at + 1..]),
        None => (dir.to_string(), prefix),
    };
    let mut matches: Vec<String> = fs::read_dir(&sub)
        .map(|read| {
            read.flatten()
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|name| name.starts_with(name_prefix))
                .filter(|name| !name.starts_with('.') || name_prefix.starts_with('.'))
                .map(|name| format!("{sub}/{name}"))
                .collect()
        })
        .unwrap_or_default();
    if matches.is_empty() {
        return vec![format!("{dir}/{prefix}*")];
    }
    matches.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    matches
}

/// `[ -x path ]`: whether this user may execute (or, for a directory, search) it.
pub fn is_executable(path: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;
    let Ok(c_path) = std::ffi::CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };
    // SAFETY: `c_path` is a valid NUL-terminated string for the whole call.
    unsafe { libc::access(c_path.as_ptr(), libc::X_OK) == 0 }
}
