//! Where Node versions and aliases live.
//!
//! Commands, messages and version resolution go through [`Backend`], so a
//! project that keeps Node installs elsewhere can reuse them. The trait asks
//! about versions, not folders: which ones are installed, where one lives,
//! which one a path belongs to, and how to add or remove one. Nothing that
//! calls it knows how the versions are laid out on disk.
//!
//! [`NvmDirBackend`] is nvm's own `$NVM_DIR` layout and the one parity is
//! measured against. A backend for another store has five things to say:
//! [`Backend::root`], [`Backend::installed`], [`Backend::version_path`],
//! [`Backend::owner_of`] and [`Backend::remove`]. The rest have defaults
//! that fit most stores. See each method for when to replace it.

use crate::context::Nvm;
use crate::nvm::{paths, version};
use crate::shell::Vars;
use std::fs;
use std::path::Path;

/// The two families of releases nvm installs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flavor {
    Node,
    Iojs,
}

impl Flavor {
    /// The name nvm gives the flavor: `node` or `iojs`.
    pub fn name(self) -> &'static str {
        match self {
            Flavor::Node => version::NODE_PREFIX,
            Flavor::Iojs => version::IOJS_PREFIX,
        }
    }
}

/// Which installs a listing looks at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flavors {
    Node,
    Iojs,
    Both,
}

/// One installed version, as a backend reports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Installed {
    pub flavor: Flavor,
    /// The version with its `v`, such as `v22.1.0`. An io.js version has no
    /// `iojs-` in front here: `v3.3.1` with [`Flavor::Iojs`].
    pub name: String,
}

/// What `nvm install` asks a backend that installs versions its own way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallRequest<'a> {
    /// The exact version, as nvm names it: `v22.1.0` or `iojs-v3.3.1`.
    pub version: &'a str,
    pub flavor: Flavor,
    /// Whether the caller asked for a build from source (`nvm install -s`).
    pub from_source: bool,
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
    /// `$NVM_DIR`: where nvm's own files are, such as the download cache
    /// and `default-packages`. Installed versions need not be inside it.
    fn root(&self, vars: &Vars) -> String;

    /// Every installed version of the flavors asked for. The order only
    /// matters between two versions with the same number, where nvm keeps
    /// the first.
    fn installed(&self, vars: &Vars, flavors: Flavors) -> Vec<Installed>;

    /// The folder a version is installed in, or would be: the one holding
    /// `bin/node`. `version` is nvm's name for it, `v22.1.0` or
    /// `iojs-v3.3.1`, and has been checked for `..` already.
    ///
    /// nvm: nvm_version_path
    fn version_path(&self, vars: &Vars, version: &str) -> String;

    /// Which flavor's installs a path lies inside, or `None` for a path
    /// that is not this backend's. `nvm current` asks this about the `node`
    /// found on `PATH`, and a `node` that is not the backend's is `system`.
    fn owner_of(&self, vars: &Vars, path: &str) -> Option<Flavor>;

    /// Removes an installed version, as `rm -rf` on its folder would.
    fn remove(&self, vars: &Vars, version: &str) -> std::io::Result<()>;

    /// Installs a version in the backend's own way and returns nvm's exit
    /// status, or returns `None` to let nvm download and unpack the release
    /// into [`Backend::version_path`] itself. What the backend wants shown
    /// goes through `nvm.io`. After a status of 0, nvm checks the install
    /// and switches to it as it does after its own.
    fn install(&self, _nvm: &mut Nvm, _request: &InstallRequest) -> Option<i32> {
        None
    }

    /// Takes this backend's entries ending in `suffix` out of a colon list
    /// such as `PATH`. The default looks for them under [`Backend::root`],
    /// in nvm's layout. A backend with every version in one folder can call
    /// [`paths::strip_path`] with that folder.
    ///
    /// nvm: nvm_strip_path
    fn strip_path(&self, vars: &Vars, list: &str, suffix: &str) -> String {
        paths::strip_path(list, &self.root(vars), suffix)
    }

    /// Puts `version_path + suffix` into a colon list in place of the entry
    /// of this backend's already there, or in front when there is none.
    /// Replace it together with [`Backend::strip_path`].
    ///
    /// nvm: nvm_change_path
    fn change_path(&self, vars: &Vars, list: &str, suffix: &str, version_path: &str) -> String {
        paths::change_path(list, suffix, version_path, &self.root(vars))
    }

    /// `[ -x "${path}" ]`.
    fn is_executable(&self, path: &str) -> bool {
        is_executable(Path::new(path))
    }

    /// The folder holding one file per alias, with LTS aliases in `lts/`.
    ///
    /// nvm: nvm_alias_path
    fn alias_dir(&self, vars: &Vars) -> String {
        format!("{}/alias", self.root(vars))
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

    /// Writes an alias file. nvm writes the target followed by a newline.
    fn write_alias(&self, vars: &Vars, name: &str, target: &str) -> std::io::Result<()> {
        fs::write(
            format!("{}/{name}", self.alias_dir(vars)),
            format!("{target}\n"),
        )
    }

    fn remove_alias(&self, vars: &Vars, name: &str) {
        let _ = fs::remove_file(format!("{}/{name}", self.alias_dir(vars)));
    }

    /// Whether `alias/<name>` is a regular file.
    fn alias_exists(&self, vars: &Vars, name: &str) -> bool {
        Path::new(&format!("{}/{name}", self.alias_dir(vars))).is_file()
    }

    /// What `"${alias_dir}/${prefix}"*` expands to, in glob order. When
    /// nothing matches, the shell leaves the pattern itself.
    fn alias_glob(&self, vars: &Vars, prefix: &str) -> Vec<String> {
        glob_prefix(&self.alias_dir(vars), prefix)
    }

    /// `mkdir -p` of a directory nvm creates as a side effect.
    fn make_dirs(&self, path: &str) {
        let _ = fs::create_dir_all(path);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
}

/// nvm's own layout under `$NVM_DIR`: Node 0.12 and later in
/// `versions/node`, io.js in `versions/io.js`, and Node before 0.12 in
/// `$NVM_DIR` itself.
pub struct NvmDirBackend;

impl NvmDirBackend {
    fn node_dir(root: &str) -> String {
        format!("{root}/versions/node")
    }

    fn iojs_dir(root: &str) -> String {
        format!("{root}/versions/io.js")
    }
}

impl Backend for NvmDirBackend {
    fn root(&self, vars: &Vars) -> String {
        vars.get("NVM_DIR").to_string()
    }

    fn installed(&self, vars: &Vars, flavors: Flavors) -> Vec<Installed> {
        let root = self.root(vars);
        let mut found = Vec::new();
        let mut look = |dir: &str, flavor: Flavor, beside_nvm: bool| {
            for entry in dir_entries(dir).into_iter().filter(|entry| entry.is_dir) {
                // Old installs sit in `$NVM_DIR` beside nvm's own folders,
                // and nvm tells them apart by the leading `v`.
                if beside_nvm && (!entry.name.starts_with('v') || entry.name == "versions") {
                    continue;
                }
                found.push(Installed {
                    flavor,
                    name: entry.name,
                });
            }
        };
        if flavors != Flavors::Iojs {
            look(&root, Flavor::Node, true);
            look(&Self::node_dir(&root), Flavor::Node, false);
        }
        if flavors != Flavors::Node {
            look(&Self::iojs_dir(&root), Flavor::Iojs, false);
        }
        found
    }

    fn version_path(&self, vars: &Vars, version: &str) -> String {
        let root = self.root(vars);
        if version::is_iojs(version) {
            format!(
                "{}/{}",
                Self::iojs_dir(&root),
                version::strip_iojs_prefix(version)
            )
        } else if version::greater("0.12.0", version) {
            format!("{root}/{version}")
        } else {
            format!("{}/{version}", Self::node_dir(&root))
        }
    }

    fn owner_of(&self, vars: &Vars, path: &str) -> Option<Flavor> {
        let root = self.root(vars);
        if paths::tree_contains_path(&Self::iojs_dir(&root), path) == Some(true) {
            Some(Flavor::Iojs)
        } else if paths::tree_contains_path(&root, path) == Some(true) {
            Some(Flavor::Node)
        } else {
            None
        }
    }

    fn remove(&self, vars: &Vars, version: &str) -> std::io::Result<()> {
        remove_tree(&self.version_path(vars, version))
    }
}

/// The entries of a directory, as `"${dir}"/*` expands: names that do not
/// start with a dot, sorted, with real directories marked.
pub fn dir_entries(dir: &str) -> Vec<Entry> {
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

/// Removes one path the way `rm -rf` does: a directory with everything in
/// it, a symlink without what it points to, nothing when there is nothing.
pub fn remove_tree(path: &str) -> std::io::Result<()> {
    if path.is_empty() {
        return Ok(());
    }
    let result = match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(error) => Err(error),
    };
    match result {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
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
