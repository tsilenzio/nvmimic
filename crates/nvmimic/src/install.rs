//! Installing nvmimic, taking over an nvm that is already in `$NVM_DIR`, and
//! putting that nvm back.
//!
//! In `$NVM_DIR` only nvm's three scripts are replaced. `versions/`, `alias/`
//! and `.cache/` are never touched, so installed Node versions keep working
//! with either. Everything else nvmimic needs lives outside `$NVM_DIR`, see
//! [`crate::dirs`].

use crate::dirs::{CACHE_KINDS, Dirs};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// What the scripts in `shell/` hold where a path of the installation goes.
/// `self install` writes the real paths in, see [`bind`].
pub const BINARY_PLACEHOLDER: &str = "\"@nvmimic-binary@\"";
const CACHE_PLACEHOLDER: &str = "\"@nvmimic-cache@\"";
const VERSION_PLACEHOLDER: &str = "\"@nvmimic-version@\"";

const NVM_SH: &str = include_str!("../../../shell/nvm.sh");
const INIT: &str = include_str!("../../../shell/init.sh");
const NVM_EXEC: &str = include_str!("../../../shell/nvm-exec");
const BASH_COMPLETION: &str = include_str!("../../../shell/bash_completion");

/// The scripts nvm keeps in `$NVM_DIR`, which nvmimic replaces.
const SCRIPTS: &[&str] = &["nvm.sh", "nvm-exec", "bash_completion"];

/// Text only nvmimic's own `nvm.sh` contains.
const OWN_MARKER: &str = "as nvmimic provides it";
/// Text only nvmimic's own `bash_completion` contains.
const COMPLETION_MARKER: &str = "Changed by nvmimic";

/// The line in the text `nvmimic init` prints where the function
/// definitions end. What follows runs once, when a shell loads nvmimic.
pub const LOAD_MARKER: &str =
    "# The functions end here. The rest runs once, when a shell loads nvmimic.\n";

/// The paths the shell functions name.
pub struct Bound<'a> {
    /// The program the functions hand their work to.
    pub binary: &'a Path,
    /// Where answers are remembered. `None` remembers nothing.
    pub cache: Option<&'a Path>,
}

/// Shell text with the installation's paths and nvmimic's version written
/// in, each path in full. nvm lets `$NVM_DIR` change after sourcing, and
/// nvmimic's own files are not in `$NVM_DIR` to begin with, so nothing can
/// be found from there.
fn bind(script: &str, paths: &Bound) -> String {
    let quoted = |path: &Path| crate::shell::quote(&path.display().to_string());
    script
        .replace(BINARY_PLACEHOLDER, &quoted(paths.binary))
        .replace(
            CACHE_PLACEHOLDER,
            &paths.cache.map_or("''".to_string(), quoted),
        )
        .replace(
            VERSION_PLACEHOLDER,
            &crate::shell::quote(env!("CARGO_PKG_VERSION")),
        )
}

/// The `nvm.sh` that goes into `$NVM_DIR`: a few lines that run `binary`
/// with `init` and `eval` what it prints.
pub fn nvm_sh(binary: &Path) -> String {
    let paths = Bound {
        binary,
        cache: None,
    };
    bind(NVM_SH, &paths)
}

/// Where nvm's own lines for a missing `NVM_DIR` start and end in what
/// `nvmimic init` prints. They take the folder `nvm.sh` was sourced from.
const FOLDER_START: &str = "  # shellcheck disable=SC2128\n";
const FOLDER_END: &str = "  export NVM_DIR\n";

/// What `nvmimic init` prints: the shell functions, then the lines that
/// load the default version. `load` takes the place of the last of those
/// lines, see [`crate::init::load`].
///
/// `nvm_dir` is for `init` typed directly, as in `eval "$(nvmimic init)"`.
/// No `nvm.sh` was sourced then, and the file being read is a startup file
/// in some other folder. So a shell without `NVM_DIR` is given `nvm_dir`
/// in place of the folder nvm would work out.
pub fn init_text(paths: &Bound, nvm_dir: Option<&Path>, load: Option<&str>) -> String {
    let mut text = bind(INIT, paths);
    if let Some(nvm_dir) = nvm_dir
        && let Some(marker) = text.find(LOAD_MARKER)
        && let Some(start) = text[marker..].find(FOLDER_START)
        && let Some(end) = text[marker + start..].find(FOLDER_END)
    {
        let start = marker + start;
        let folder = crate::shell::quote(&nvm_dir.display().to_string());
        text.replace_range(start..start + end, &format!("  NVM_DIR={folder}\n"));
    }
    match (load, text.strip_suffix(crate::init::SECOND_START)) {
        (Some(load), Some(head)) => format!("{head}{load}"),
        _ => text,
    }
}

/// The shell functions alone, for a shell that already loaded nvmimic and
/// holds an earlier version's.
pub fn functions_text(paths: &Bound) -> String {
    let functions = INIT.split_once(LOAD_MARKER).map_or(INIT, |(head, _)| head);
    bind(functions, paths)
}

/// What the installer talks to: lines to show, and yes-or-no questions.
pub trait Ui {
    fn say(&mut self, line: &str);
    /// Asks a yes-or-no question. `default` is the answer an empty reply gives.
    fn ask(&mut self, question: &str, default: bool) -> bool;
}

#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Don't ask before going ahead.
    pub force: bool,
    /// Say what would happen and change nothing.
    pub dry_run: bool,
    /// `self uninstall --purge`: delete all of `$NVM_DIR` as well.
    pub purge: bool,
    /// Whether there is a terminal to ask questions at.
    pub terminal: bool,
}

/// The parts of the environment the installer looks at.
#[derive(Clone, Debug)]
pub struct Env {
    pub home: PathBuf,
    pub nvm_dir: Option<String>,
    pub xdg_config_home: Option<String>,
    pub homebrew_prefix: Option<String>,
    pub cargo_home: Option<String>,
    pub path: Option<String>,
    /// Where nvmimic's own files go. `None` without a home folder.
    pub dirs: Option<Dirs>,
    /// Whether `NVMIMIC_INSTALL_PATH` says where the program goes.
    pub install_path_given: bool,
}

impl Env {
    pub fn from_process() -> Env {
        let var = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
        Env {
            home: var("HOME").map(PathBuf::from).unwrap_or_default(),
            nvm_dir: var("NVM_DIR"),
            xdg_config_home: var("XDG_CONFIG_HOME"),
            homebrew_prefix: var("HOMEBREW_PREFIX"),
            cargo_home: var("CARGO_HOME"),
            path: var("PATH"),
            dirs: Dirs::resolve(var),
            install_path_given: var("NVMIMIC_INSTALL_PATH").is_some(),
        }
    }

    /// Where nvm lives, the way nvm's own installer picks it.
    pub fn nvm_dir(&self) -> PathBuf {
        if let Some(dir) = &self.nvm_dir {
            return PathBuf::from(dir.trim_end_matches('/'));
        }
        match &self.xdg_config_home {
            Some(config) => Path::new(config).join("nvm"),
            None => self.home.join(".nvm"),
        }
    }

    pub(crate) fn dirs(&self) -> Result<&Dirs, String> {
        self.dirs.as_ref().ok_or_else(|| {
            "there is no home folder (HOME) to keep nvmimic's own files in.".to_string()
        })
    }

    /// Homebrew's nvm, if it is installed: the prefix it lives under.
    fn homebrew_nvm(&self) -> Option<PathBuf> {
        let mut prefixes: Vec<PathBuf> = Vec::new();
        if let Some(prefix) = &self.homebrew_prefix {
            prefixes.push(prefix.into());
        }
        for prefix in ["/opt/homebrew", "/usr/local", "/home/linuxbrew/.linuxbrew"] {
            prefixes.push(prefix.into());
        }
        prefixes
            .into_iter()
            .find(|prefix| prefix.join("opt/nvm").exists())
    }
}

/// The record of what an install changed, kept as `manifest` in nvmimic's
/// state folder.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Manifest {
    pub version: String,
    /// The `$NVM_DIR` nvmimic was installed for.
    pub nvm_dir: Option<PathBuf>,
    /// The program the installed `nvm.sh` runs.
    pub binary: Option<PathBuf>,
    /// Whether `self install` put the program there. If so, `self uninstall`
    /// takes it away again.
    pub placed: bool,
    pub data: Option<PathBuf>,
    pub cache: Option<PathBuf>,
    /// Scripts that were nvm's, with a copy of each in the backup folder.
    pub backed_up: Vec<String>,
    /// Scripts that were links when nvmimic took over, and where each one
    /// pointed. A link goes back only while its target is still there.
    pub links: BTreeMap<String, PathBuf>,
    /// Scripts nvmimic added where there were none.
    pub created: Vec<String>,
    /// What each script held when nvmimic wrote it, as a SHA-256 sum. A file
    /// with another sum was changed by someone since.
    pub fingerprints: BTreeMap<String, String>,
    /// `self eject` took nvmimic's scripts out of `$NVM_DIR` and left the
    /// program installed.
    pub ejected: bool,
    /// The version of the earlier program `self update` set aside, which
    /// `self rollback` brings back.
    pub previous: Option<String>,
}

impl Manifest {
    pub(crate) fn render(&self) -> String {
        let mut text = String::from(
            "# What `nvmimic self install` changed. `nvmimic self uninstall` reads it.\n",
        );
        text.push_str(&format!("version {}\n", self.version));
        for (key, path) in [
            ("nvm-dir", &self.nvm_dir),
            ("binary", &self.binary),
            ("data", &self.data),
            ("cache", &self.cache),
        ] {
            if let Some(path) = path {
                text.push_str(&format!("{key} {}\n", path.display()));
            }
        }
        if self.placed {
            text.push_str("placed binary\n");
        }
        for name in &self.backed_up {
            text.push_str(&format!("backup {name}\n"));
        }
        for (name, target) in &self.links {
            text.push_str(&format!("link {name} {}\n", target.display()));
        }
        for name in &self.created {
            text.push_str(&format!("created {name}\n"));
        }
        for (name, sum) in &self.fingerprints {
            text.push_str(&format!("fingerprint {name} {sum}\n"));
        }
        if self.ejected {
            text.push_str("ejected scripts\n");
        }
        if let Some(version) = &self.previous {
            text.push_str(&format!("previous {version}\n"));
        }
        text
    }

    fn parse(text: &str) -> Manifest {
        let mut manifest = Manifest::default();
        for line in text.lines().filter(|line| !line.starts_with('#')) {
            let Some((key, value)) = line.split_once(' ') else {
                continue;
            };
            let path = || Some(PathBuf::from(value));
            match key {
                "version" => manifest.version = value.to_string(),
                "nvm-dir" => manifest.nvm_dir = path(),
                "binary" => manifest.binary = path(),
                "data" => manifest.data = path(),
                "cache" => manifest.cache = path(),
                "placed" => manifest.placed = true,
                "backup" => manifest.backed_up.push(value.to_string()),
                "link" => {
                    if let Some((name, target)) = value.split_once(' ') {
                        manifest.links.insert(name.to_string(), target.into());
                    }
                }
                "created" => manifest.created.push(value.to_string()),
                "fingerprint" => {
                    if let Some((name, sum)) = value.split_once(' ') {
                        manifest
                            .fingerprints
                            .insert(name.to_string(), sum.to_string());
                    }
                }
                "ejected" => manifest.ejected = true,
                "previous" => manifest.previous = Some(value.to_string()),
                _ => {}
            }
        }
        manifest
    }

    /// The manifest of the installation in `dirs`, if there is one.
    pub fn read(dirs: &Dirs) -> Option<Manifest> {
        fs::read_to_string(dirs.manifest())
            .ok()
            .map(|text| Manifest::parse(&text))
    }
}

/// A package manager that installs and upgrades the nvmimic program itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Managed {
    Homebrew,
    Cargo,
}

/// Which package manager owns the program at `binary`, if one does.
pub fn managed_by(binary: &Path, env: &Env) -> Option<Managed> {
    let real = fs::canonicalize(binary).ok()?;
    if real.components().any(|part| part.as_os_str() == "Cellar") {
        return Some(Managed::Homebrew);
    }
    let cargo_bin = match &env.cargo_home {
        Some(cargo_home) => Path::new(cargo_home).join("bin"),
        None => env.home.join(".cargo/bin"),
    };
    (real.parent() == fs::canonicalize(cargo_bin).ok().as_deref()).then_some(Managed::Cargo)
}

/// The path that keeps naming a package manager's program after an upgrade.
/// Homebrew keeps each version in a folder of its own and links the newest
/// one from `<prefix>/bin`.
fn lasting_path(binary: &Path) -> PathBuf {
    let Ok(real) = fs::canonicalize(binary) else {
        return binary.to_path_buf();
    };
    let prefix = real
        .ancestors()
        .find(|dir| dir.file_name().is_some_and(|name| name == "Cellar"))
        .and_then(Path::parent);
    match prefix.map(|prefix| prefix.join("bin/nvmimic")) {
        Some(linked) if fs::canonicalize(&linked).ok().as_ref() == Some(&real) => linked,
        _ => binary.to_path_buf(),
    }
}

fn is_ours(path: &Path) -> bool {
    fs::read_to_string(path).is_ok_and(|text| text.contains(OWN_MARKER))
}

/// Whether a file is present, a dangling symlink included.
fn present(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

/// Whether `path` is a link whose target is gone.
fn dead_link(path: &Path) -> bool {
    present(path) && fs::metadata(path).is_err()
}

/// The name a file is written under before it is renamed into place.
fn unfinished(path: &Path) -> PathBuf {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    path.with_file_name(format!(".{name}.nvmimic-new"))
}

/// Copies what `from` holds to `to`. A link is followed, so the copy is a
/// file of its own. `to` is complete or not there at all, never half
/// written.
fn keep_copy(from: &Path, to: &Path) -> Result<(), String> {
    let temporary = unfinished(to);
    fs::copy(from, &temporary).map_err(|e| format!("{}: {e}", from.display()))?;
    fs::rename(&temporary, to).map_err(|e| format!("{}: {e}", to.display()))
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// How a question shows which answer an empty reply gives.
fn choices(default: bool) -> &'static str {
    if default { "[Y/n]" } else { "[y/N]" }
}

pub(crate) fn ask(ui: &mut dyn Ui, options: &Options, question: &str, default: bool) -> bool {
    if options.force {
        ui.say(&format!("{question} {} y (--force)", choices(default)));
        true
    } else {
        ui.ask(question, default)
    }
}

/// Where the nvmimic program ends up.
enum BinaryPlan {
    /// A package manager owns the program, so it stays where that put it.
    Managed(PathBuf),
    /// The program being run already is the installed one.
    Keep(PathBuf),
    Copy(PathBuf),
}

impl BinaryPlan {
    fn path(&self) -> &Path {
        match self {
            BinaryPlan::Managed(path) | BinaryPlan::Keep(path) | BinaryPlan::Copy(path) => path,
        }
    }
}

fn plan_binary(
    env: &Env,
    dirs: &Dirs,
    running: &Path,
    manifest: &Manifest,
) -> Result<BinaryPlan, String> {
    if !env.install_path_given && managed_by(running, env).is_some() {
        return Ok(BinaryPlan::Managed(lasting_path(running)));
    }
    let target = dirs.binary.clone();
    if same_file(running, &target) {
        return Ok(BinaryPlan::Keep(target));
    }
    let ours = manifest.placed && manifest.binary.as_deref() == Some(&target);
    if present(&target) && !ours {
        return Err(format!(
            "{} is already there and `nvmimic self install` didn't put it there, so it is left \
             alone. Move it away, or set NVMIMIC_INSTALL_PATH to the place the nvmimic program \
             should go.",
            target.display()
        ));
    }
    Ok(BinaryPlan::Copy(target))
}

#[cfg(test)]
thread_local! {
    /// The step of an install to stop before, and the steps taken so far.
    static STOP_BEFORE: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
    static STEPS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Called before each change `self install` makes on disk. A test can make
/// one of these calls fail, which leaves the disk the way a crash or a
/// power cut at that moment would.
fn checkpoint() -> Result<(), String> {
    #[cfg(test)]
    {
        let step = STEPS.get();
        STEPS.set(step + 1);
        if STOP_BEFORE.get() == Some(step) {
            return Err("interrupted".to_string());
        }
    }
    Ok(())
}

/// `nvmimic self install`. `running` is the program being run.
pub fn self_install(
    env: &Env,
    running: &Path,
    options: &Options,
    ui: &mut dyn Ui,
) -> Result<(), String> {
    let dirs = env.dirs()?;
    let nvm_dir = env.nvm_dir();
    let backup = dirs.backup();
    let previous = Manifest::read(dirs);
    let reinstall = previous.is_some();
    let mut manifest = previous.unwrap_or_default();
    if let Some(installed_for) = &manifest.nvm_dir
        && *installed_for != nvm_dir
    {
        return Err(format!(
            "nvmimic is already installed for {}, and NVM_DIR now names {}. Run `nvmimic self \
             uninstall` first if you want to move it.",
            installed_for.display(),
            nvm_dir.display()
        ));
    }
    let plan = plan_binary(env, dirs, running, &manifest)?;

    let homebrew = env.homebrew_nvm();
    if let Some(prefix) = &homebrew {
        ui.say(&format!(
            "Homebrew's nvm is installed at {}.",
            prefix.join("opt/nvm").display()
        ));
        ui.say("Shells that load Homebrew's nvm.sh would keep running real nvm, so it has to");
        ui.say(
            "go before nvmimic can take over. Your installed Node versions stay where they are.",
        );
        for file in startup_files_mentioning(env, "opt/nvm/nvm.sh") {
            ui.say(&format!(
                "  {} loads Homebrew's nvm. Change it to load $NVM_DIR/nvm.sh.",
                file.display()
            ));
        }
        if !ask(ui, options, "Run `brew uninstall nvm` now?", false) {
            return Err("stopped. Homebrew's nvm is still installed.".to_string());
        }
        if options.dry_run {
            ui.say("Would run `brew uninstall nvm`.");
        }
    }

    // nvm-exec is the same file in nvm and nvmimic, so `nvm.sh` decides
    // whose scripts these are: while it is nvmimic's, all three are.
    let existing: Vec<&str> = SCRIPTS
        .iter()
        .copied()
        .filter(|name| present(&nvm_dir.join(name)) && !is_ours(&nvm_dir.join("nvm.sh")))
        .collect();
    if !existing.is_empty() && !reinstall {
        ui.say(&format!("Found nvm in {}.", nvm_dir.display()));
        ui.say("nvmimic aims to be a drop-in replacement: it uses the Node versions, aliases");
        ui.say("and settings that are already there, and only swaps out nvm's scripts.");
        ui.say(&format!(
            "The scripts it replaces ({}) are kept in {} and come back with `nvmimic self uninstall`.",
            existing.join(", "),
            backup.display()
        ));
        if nvm_dir.join(".git").exists() {
            ui.say(
                "This nvm was installed with git. While nvmimic is installed, `git status` in that",
            );
            ui.say("folder shows the swapped scripts as modified. That is expected and harmless.");
            ui.say("Running nvm's install script again puts nvm's scripts back over nvmimic's. If that");
            ui.say("happens, run `nvmimic self install` again.");
        }
        ui.say("Making a full backup of the folder first is a good idea.");
        if ask(
            ui,
            options,
            &format!(
                "Copy all of {} to a backup folder first?",
                nvm_dir.display()
            ),
            false,
        ) {
            let target = full_backup_path(&nvm_dir);
            if options.dry_run {
                ui.say(&format!(
                    "Would copy {} to {}.",
                    nvm_dir.display(),
                    target.display()
                ));
            } else {
                copy_dir(&nvm_dir, &target)?;
                ui.say(&format!(
                    "Copied {} to {}.",
                    nvm_dir.display(),
                    target.display()
                ));
            }
        }
        if !ask(ui, options, "Install nvmimic now?", false) {
            return Err("stopped. Nothing was changed.".to_string());
        }
    }

    if existing.is_empty() && !reinstall {
        ui.say(&format!(
            "There is no nvm in {} yet, so nvmimic sets one up from scratch.",
            nvm_dir.display()
        ));
    }

    if !existing.is_empty() && manifest.ejected {
        ui.say(&format!(
            "Taking over from the nvm in {} again.",
            nvm_dir.display()
        ));
    } else if !existing.is_empty() && reinstall {
        ui.say(&format!(
            "nvm's own {} came back in {}, most likely from nvm's install script.",
            existing.join(", "),
            nvm_dir.display()
        ));
        ui.say("Taking over again. They replace the older copies in the backup folder.");
    }

    if options.dry_run {
        ui.say(&format!("Would install nvmimic for {}.", nvm_dir.display()));
        for name in &existing {
            ui.say(&format!(
                "Would keep a copy of {} in {}.",
                nvm_dir.join(name).display(),
                backup.display()
            ));
        }
        match &plan {
            BinaryPlan::Copy(target) => {
                ui.say(&format!("Would copy this program to {}.", target.display()))
            }
            BinaryPlan::Keep(target) | BinaryPlan::Managed(target) => {
                ui.say(&format!("Would use the program at {}.", target.display()))
            }
        }
        return Ok(());
    }

    // From here on the order is what keeps a stop at any moment harmless,
    // whether it is a crash, a power cut or Ctrl-C. nvm's scripts are copied
    // first, and the manifest that names the copies is written before
    // anything of nvm's is replaced or uninstalled. So each of nvm's scripts
    // is always in `$NVM_DIR` or in a backup the manifest knows about, and
    // `self install` and `self uninstall` both work on whatever a stop left.
    for dir in [&nvm_dir, &dirs.data, &backup, &dirs.cache] {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let mut copied: Vec<&str> = Vec::new();
    for name in SCRIPTS {
        let path = nvm_dir.join(name);
        if existing.contains(name) {
            // Homebrew's nvm puts links into `$NVM_DIR` that point into its
            // own folder, and `brew uninstall nvm` deletes what they point
            // to. The copy holds the script itself, taken before that.
            match fs::read_link(&path) {
                Ok(target) => manifest.links.insert(name.to_string(), target),
                Err(_) => manifest.links.remove(*name),
            };
            if !dead_link(&path) {
                checkpoint()?;
                keep_copy(&path, &backup.join(name))?;
                copied.push(name);
                if !manifest.backed_up.iter().any(|n| n == name) {
                    manifest.backed_up.push(name.to_string());
                }
            }
            // A script nvm has since put there is nvm's to keep on uninstall.
            manifest.created.retain(|n| n != name);
        } else if !present(&path) && !manifest.created.iter().any(|n| n == name) {
            manifest.created.push(name.to_string());
        }
    }

    let scripts: Vec<(&str, String)> = SCRIPTS
        .iter()
        .map(|name| (*name, script_text(name, plan.path())))
        .collect();
    for (name, text) in &scripts {
        manifest
            .fingerprints
            .insert(name.to_string(), fingerprint(text.as_bytes()));
    }
    manifest.ejected = false;
    manifest.version = env!("CARGO_PKG_VERSION").to_string();
    manifest.nvm_dir = Some(nvm_dir.clone());
    manifest.placed = match &plan {
        BinaryPlan::Copy(_) => true,
        BinaryPlan::Keep(_) => manifest.placed,
        BinaryPlan::Managed(_) => false,
    };
    manifest.binary = Some(plan.path().to_path_buf());
    manifest.data = Some(dirs.data.clone());
    manifest.cache = Some(dirs.cache.clone());
    checkpoint()?;
    write_file(&dirs.manifest(), &manifest.render(), 0o644)?;

    if let Some(prefix) = &homebrew {
        let brew = prefix.join("bin/brew");
        let brew = if brew.is_file() {
            brew
        } else {
            PathBuf::from("brew")
        };
        let status = Command::new(&brew).args(["uninstall", "nvm"]).status();
        if !status.is_ok_and(|status| status.success()) {
            // Nothing of nvm's was touched yet. A first install leaves no
            // record behind either.
            if !reinstall {
                for name in copied {
                    let _ = fs::remove_file(backup.join(name));
                }
                let _ = fs::remove_file(dirs.manifest());
            }
            return Err("`brew uninstall nvm` did not finish, so nothing was changed.".to_string());
        }
    }

    ui.say(&format!("Installing nvmimic for {}.", nvm_dir.display()));
    if let BinaryPlan::Copy(target) = &plan {
        checkpoint()?;
        install_binary(running, target)?;
        ui.say(&format!("The nvmimic program is at {}.", target.display()));
    }
    for (name, text) in &scripts {
        let mode = if *name == "nvm-exec" { 0o755 } else { 0o644 };
        checkpoint()?;
        write_file(&nvm_dir.join(name), text, mode)?;
    }

    ui.say("Done.");
    if let Some(bin_dir) = plan.path().parent()
        && !on_path(env, bin_dir)
    {
        ui.say(&format!(
            "{} isn't on your PATH yet. Until it is, type {} to run nvmimic.",
            bin_dir.display(),
            plan.path().display()
        ));
    }
    if startup_files_mentioning(env, "/nvm.sh").is_empty() {
        ui.say(
            "To get the `nvm` command in new shells, add these lines to your shell's startup file:",
        );
        ui.say(&format!("  export NVM_DIR=\"{}\"", nvm_dir.display()));
        ui.say("  [ -s \"$NVM_DIR/nvm.sh\" ] && \\. \"$NVM_DIR/nvm.sh\"  # This loads nvm");
    } else {
        ui.say("Open a new shell to start using it.");
    }
    Ok(())
}

/// A short name for what a script holds. The manifest keeps one per script,
/// so a later run can tell whether someone changed the file.
fn fingerprint(contents: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(contents)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// What nvmimic writes as script `name` for the program at `binary`.
fn script_text(name: &str, binary: &Path) -> String {
    match name {
        "nvm.sh" => nvm_sh(binary),
        "nvm-exec" => NVM_EXEC.to_string(),
        _ => BASH_COMPLETION.to_string(),
    }
}

/// Whose a script in `$NVM_DIR` is, going by what it holds.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Found {
    Missing,
    /// What nvmimic wrote, untouched.
    Ours,
    /// nvmimic's script with someone's changes in it.
    OursEdited,
    /// Anything else. The usual way to get here is running nvm's own
    /// installer again, which writes nvm's scripts over nvmimic's.
    NotOurs,
}

/// Text only nvmimic's copy of a script holds. nvmimic's `nvm-exec` is
/// nvm's own file, byte for byte, so it has none.
fn own_marker(name: &str) -> Option<&'static str> {
    match name {
        "nvm.sh" => Some(OWN_MARKER),
        "bash_completion" => Some(COMPLETION_MARKER),
        _ => None,
    }
}

/// Looks at the script at `path`. Without a fingerprint on record, it is
/// compared with what this program would write now. A changed file only
/// counts as nvmimic's when it carries nvmimic's marker. Everything that
/// can't be told apart for sure is `NotOurs`, which gets left alone.
fn found(path: &Path, name: &str, manifest: &Manifest, binary: &Path) -> Found {
    let Ok(contents) = fs::read(path) else {
        return Found::Missing;
    };
    let written = match manifest.fingerprints.get(name) {
        Some(sum) => sum.clone(),
        None => fingerprint(script_text(name, binary).as_bytes()),
    };
    if fingerprint(&contents) == written {
        return Found::Ours;
    }
    let text = String::from_utf8_lossy(&contents);
    match own_marker(name) {
        Some(marker) if text.contains(marker) => Found::OursEdited,
        _ => Found::NotOurs,
    }
}

/// Where an edited script goes: `<script>.nvmimic-saved`, with a number
/// after it when an earlier one is still there.
fn saved_path(script: &Path) -> PathBuf {
    let name = script
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let first = script.with_file_name(format!("{name}.nvmimic-saved"));
    if !present(&first) {
        return first;
    }
    (2..)
        .map(|n| script.with_file_name(format!("{name}.nvmimic-saved.{n}")))
        .find(|path| !present(path))
        .unwrap()
}

/// The size of everything in `dir`, in bytes. Links are not followed.
fn folder_size(dir: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| match fs::symlink_metadata(entry.path()) {
            Ok(meta) if meta.is_dir() => folder_size(&entry.path()),
            Ok(meta) => meta.len(),
            Err(_) => 0,
        })
        .sum()
}

/// A size the way macOS shows it, in steps of 1000.
fn human_size(bytes: u64) -> String {
    let mut size = bytes as f64;
    for unit in ["bytes", "KB", "MB"] {
        if size < 1000.0 {
            return match unit {
                "bytes" => format!("{bytes} bytes"),
                _ => format!("{size:.0} {unit}"),
            };
        }
        size /= 1000.0;
    }
    format!("{size:.1} GB")
}

/// `--purge` deletes a whole folder, so a `$NVM_DIR` that is the home
/// folder, or holds it, is refused.
fn purge_allowed(nvm_dir: &Path, env: &Env) -> Result<(), String> {
    let real = fs::canonicalize(nvm_dir).unwrap_or_else(|_| nvm_dir.to_path_buf());
    let home = fs::canonicalize(&env.home).unwrap_or_else(|_| env.home.clone());
    if !real.is_absolute() || real.parent().is_none() || home.starts_with(&real) {
        return Err(format!(
            "--purge would delete {}, and that isn't a folder nvmimic will delete. Nothing was changed.",
            nvm_dir.display()
        ));
    }
    Ok(())
}

/// The three ways nvmimic leaves `$NVM_DIR`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Leaving {
    /// nvm's scripts go back and the nvmimic program stays.
    Eject,
    /// nvm's scripts go back and nvmimic goes.
    Uninstall,
    /// nvmimic goes, and all of `$NVM_DIR` with it.
    Purge,
}

/// One thing a removal does. `action` is worded to follow "Would" and
/// "This will".
struct Step {
    note: Option<String>,
    action: String,
}

impl Step {
    fn new(action: String) -> Step {
        Step { note: None, action }
    }

    fn line(&self, lead: &str) -> String {
        match &self.note {
            Some(note) => format!("{note} {lead} {}.", self.action),
            None => format!("{lead} {}.", self.action),
        }
    }
}

/// What eject and uninstall say about a script that was a link with nothing
/// behind it: a dead link is never put back.
fn nothing_behind(name: &str, target: &Path) -> String {
    format!(
        "{name} was a link to {}, and nothing was there to keep a copy of, so there is nothing to put back for it.",
        target.display()
    )
}

/// `nvmimic self eject`.
pub fn self_eject(env: &Env, options: &Options, ui: &mut dyn Ui) -> Result<(), String> {
    leave(env, options, ui, Leaving::Eject)
}

/// `nvmimic self uninstall`, with or without `--purge`.
pub fn self_uninstall(env: &Env, options: &Options, ui: &mut dyn Ui) -> Result<(), String> {
    let how = if options.purge {
        Leaving::Purge
    } else {
        Leaving::Uninstall
    };
    leave(env, options, ui, how)
}

fn leave(env: &Env, options: &Options, ui: &mut dyn Ui, how: Leaving) -> Result<(), String> {
    let dirs = env.dirs()?;
    let Some(mut manifest) = Manifest::read(dirs) else {
        return Err(format!(
            "nvmimic is not installed. There is no record of an install in {}.",
            dirs.state.display()
        ));
    };
    let nvm_dir = manifest.nvm_dir.clone().unwrap_or_else(|| env.nvm_dir());
    let backup = dirs.backup();
    let binary = manifest
        .binary
        .clone()
        .unwrap_or_else(|| dirs.binary.clone());
    let data = manifest.data.clone().unwrap_or_else(|| dirs.data.clone());
    let cache = manifest.cache.clone().unwrap_or_else(|| dirs.cache.clone());
    let placed = manifest.placed.then(|| binary.clone());
    // A backup someone deleted can't go back, and neither can one that is
    // a link to nothing, which is what an earlier nvmimic kept of a link
    // into Homebrew.
    let mut restore: Vec<String> = manifest
        .backed_up
        .iter()
        .filter(|name| fs::metadata(backup.join(name)).is_ok())
        .cloned()
        .collect();
    // Scripts that were links with nothing behind them, and where each
    // pointed. There is nothing of nvm's to put back for these.
    let mut dead: Vec<(&str, PathBuf)> = SCRIPTS
        .iter()
        .filter(|name| !restore.iter().any(|kept| kept == *name))
        .filter_map(|name| {
            let kept = backup.join(name);
            match dead_link(&kept) {
                true => fs::read_link(&kept).ok(),
                false => manifest.links.get(*name).cloned(),
            }
            .map(|target| (*name, target))
        })
        .collect();

    if how == Leaving::Eject {
        if manifest.ejected {
            return Err(format!(
                "nvmimic's scripts are already out of {}. Nothing was changed.",
                nvm_dir.display()
            ));
        }
        if restore.is_empty() && !dead.is_empty() {
            let links: Vec<String> = dead
                .iter()
                .map(|(name, target)| nothing_behind(name, target))
                .collect();
            return Err(format!(
                "there is no nvm to go back to. {} Nothing was changed.",
                links.join(" ")
            ));
        }
        if restore.is_empty() {
            return Err(format!(
                "there is no nvm to go back to. nvmimic found none in {} when it was installed, \
                 so it has no scripts of nvm's to put back. Nothing was changed.",
                nvm_dir.display()
            ));
        }
    }
    if how == Leaving::Purge {
        purge_allowed(&nvm_dir, env)?;
    }

    let mut steps: Vec<Step> = Vec::new();
    // Scripts someone changed since nvmimic wrote them: name, path, new path.
    let mut edited: Vec<(&str, PathBuf, PathBuf)> = Vec::new();
    // Scripts that aren't nvmimic's any more. They stay as they are, and
    // nvm's older copy in the backup folder doesn't go back over them.
    let mut not_ours: Vec<&str> = Vec::new();
    if how == Leaving::Purge {
        steps.push(Step::new(format!(
            "delete {} and everything in it, your Node versions and aliases included ({})",
            nvm_dir.display(),
            human_size(folder_size(&nvm_dir))
        )));
        if !restore.is_empty() {
            steps.push(Step::new(format!(
                "delete the backup of nvm's own {} in {}",
                restore.join(", "),
                backup.display()
            )));
        }
    } else if !manifest.ejected {
        // With an `nvm.sh` that isn't nvmimic's, nobody can say whose
        // `nvm-exec` is: nvm's and nvmimic's are the same file.
        let nvm_sh_is_ours =
            found(&nvm_dir.join("nvm.sh"), "nvm.sh", &manifest, &binary) != Found::NotOurs;
        for name in SCRIPTS {
            let path = nvm_dir.join(name);
            let state = match found(&path, name, &manifest, &binary) {
                Found::Ours if *name == "nvm-exec" && !nvm_sh_is_ours => Found::NotOurs,
                state => state,
            };
            match state {
                Found::OursEdited => {
                    let saved = saved_path(&path);
                    steps.push(Step {
                        note: Some(format!("{name} was edited after nvmimic installed it.")),
                        action: format!("rename {} to {}", path.display(), saved.display()),
                    });
                    edited.push((name, path, saved));
                }
                Found::NotOurs => {
                    steps.push(Step {
                        note: Some(format!(
                            "{} isn't the file nvmimic put there.",
                            path.display()
                        )),
                        action: "leave it alone".to_string(),
                    });
                    not_ours.push(name);
                }
                Found::Ours | Found::Missing => {}
            }
        }
        restore.retain(|name| !not_ours.contains(&name.as_str()));
        dead.retain(|(name, _)| !not_ours.contains(name));
        if not_ours.len() < SCRIPTS.len() {
            steps.push(Step::new(format!(
                "remove nvmimic's scripts from {}",
                nvm_dir.display()
            )));
        }
        if !restore.is_empty() {
            steps.push(Step::new(format!(
                "put nvm's own {} back in {}",
                restore.join(", "),
                nvm_dir.display()
            )));
        }
    }
    if how != Leaving::Eject {
        steps.push(Step::new(format!(
            "remove nvmimic's own files from {} and {}",
            dirs.state.display(),
            cache.display()
        )));
        if let Some(binary) = &placed {
            steps.push(Step::new(format!(
                "remove the nvmimic program at {}",
                binary.display()
            )));
        }
    }

    match how {
        Leaving::Purge => {}
        _ if manifest.ejected => ui.say(&format!(
            "nvmimic's scripts are already out of {}.",
            nvm_dir.display()
        )),
        _ if restore.is_empty() && !not_ours.is_empty() => ui.say(&format!(
            "{} holds nvm's own scripts again, so there is nothing to put back.",
            nvm_dir.display()
        )),
        Leaving::Eject => ui.say(&format!(
            "This puts nvm's own {} back and keeps the nvmimic program, so `nvmimic self install` can take over again.",
            restore.join(", ")
        )),
        _ if !restore.is_empty() => ui.say(&format!(
            "This puts nvm's own {} back, so nvm works as it did before nvmimic.",
            restore.join(", ")
        )),
        _ => ui.say(&format!(
            "There was no nvm in {} before nvmimic, so afterwards there is no `nvm` command.",
            nvm_dir.display()
        )),
    }
    if how != Leaving::Purge {
        for (name, target) in &dead {
            ui.say(&nothing_behind(name, target));
        }
        ui.say("Your Node versions and aliases stay as they are.");
    }

    if options.dry_run {
        for step in &steps {
            ui.say(&step.line("Would"));
        }
        return Ok(());
    }
    if !options.force {
        if !options.terminal {
            for step in &steps {
                ui.say(&step.line("Would"));
            }
            return Err(
                "nothing was changed. This asks before it goes ahead, and there is no terminal \
                 to ask at. Run it again with --force."
                    .to_string(),
            );
        }
        for step in &steps {
            ui.say(&step.line("This will"));
        }
        let question = match how {
            Leaving::Eject => format!("Put nvm back in {}?", nvm_dir.display()),
            Leaving::Uninstall => format!("Remove nvmimic from {}?", nvm_dir.display()),
            Leaving::Purge => format!("Delete {} for good?", nvm_dir.display()),
        };
        if !ui.ask(&question, false) {
            return Err("stopped. Nothing was changed.".to_string());
        }
    }

    if how == Leaving::Purge {
        match fs::remove_dir_all(&nvm_dir) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => {
                return Err(format!("{}: {error}", nvm_dir.display()));
            }
            _ => {}
        }
        for name in &manifest.backed_up {
            let _ = fs::remove_file(backup.join(name));
        }
    } else if !manifest.ejected {
        for (name, path, saved) in &edited {
            fs::rename(path, saved).map_err(|e| format!("{}: {e}", path.display()))?;
            ui.say(&format!(
                "{name} was edited after nvmimic installed it. Renamed {} to {}.",
                path.display(),
                saved.display()
            ));
        }
        for name in &not_ours {
            let kept = backup.join(name);
            let older = match present(&kept) {
                true => format!(" nvmimic's older copy of it stays at {}.", kept.display()),
                false => String::new(),
            };
            ui.say(&format!(
                "{} isn't the file nvmimic put there. Left it alone.{older}",
                nvm_dir.join(name).display()
            ));
        }
        for name in SCRIPTS {
            if !edited.iter().any(|(edited, _, _)| edited == name) && !not_ours.contains(name) {
                let _ = fs::remove_file(nvm_dir.join(name));
            }
        }
        for name in &restore {
            let kept = backup.join(name);
            let path = nvm_dir.join(name);
            match manifest.links.get(name) {
                // A link goes back as the link it was while its target is
                // still there. `join` leaves a full path as it is.
                Some(target) if fs::metadata(nvm_dir.join(target)).is_ok() => {
                    std::os::unix::fs::symlink(target, &path)
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                    let _ = fs::remove_file(&kept);
                }
                Some(target) => {
                    move_file(&kept, &path)?;
                    ui.say(&format!(
                        "{name} was a link to {}, which is gone. It comes back as a file with what the link pointed to.",
                        target.display()
                    ));
                }
                None => move_file(&kept, &path)?,
            }
        }
        for name in SCRIPTS {
            let kept = backup.join(name);
            if dead_link(&kept) {
                let _ = fs::remove_file(&kept);
            }
            // What a stopped install can leave half written.
            let _ = fs::remove_file(unfinished(&kept));
            let _ = fs::remove_file(unfinished(&nvm_dir.join(name)));
        }
    }

    if how == Leaving::Eject {
        manifest.backed_up.clear();
        manifest.links.clear();
        manifest.created.clear();
        manifest.fingerprints.clear();
        manifest.ejected = true;
        write_file(&dirs.manifest(), &manifest.render(), 0o644)?;
        let _ = fs::remove_dir(&backup);
        ui.say(&format!(
            "Done. nvm is back in {}, and the nvmimic program stays at {}.",
            nvm_dir.display(),
            binary.display()
        ));
        return Ok(());
    }

    // Only what nvmimic put into its folders is removed, file by file. A
    // folder that then still holds something of yours stays. The two files
    // in the data folder are what earlier versions kept there.
    for file in ["nvm-functions.sh", "nvmimic-functions.sh"] {
        let _ = fs::remove_file(data.join(file));
    }
    let _ = fs::remove_dir(&data);
    for kind in CACHE_KINDS {
        let _ = fs::remove_dir_all(cache.join(kind));
    }
    let _ = fs::remove_dir(&cache);
    if let Some(binary) = &placed {
        let _ = fs::remove_file(binary);
    }
    let _ = fs::remove_file(dirs.manifest());
    let _ = fs::remove_file(unfinished(&dirs.manifest()));
    if let Some(version) = &manifest.previous {
        let _ = fs::remove_file(crate::update::previous_path(dirs, version));
    }
    let _ = fs::remove_dir(dirs.state.join("previous"));
    let _ = fs::remove_dir(&backup);
    let _ = fs::remove_dir(&dirs.state);

    ui.say(match how {
        Leaving::Purge => "Done. nvmimic and the nvm folder are gone.",
        _ if !restore.is_empty() || manifest.ejected || !not_ours.is_empty() => {
            "Done. nvmimic is gone and nvm is back."
        }
        _ => "Done. nvmimic is gone.",
    });
    if placed.is_none() {
        let remove = match managed_by(&binary, env) {
            Some(Managed::Homebrew) => Some(("Homebrew", "brew uninstall nvmimic")),
            Some(Managed::Cargo) => Some(("cargo", "cargo uninstall nvmimic-cli")),
            None => None,
        };
        match remove {
            Some((manager, command)) => {
                ui.say(&format!(
                    "The nvmimic program stays at {}, because {manager} installed it. To remove it, run:",
                    binary.display()
                ));
                ui.say(&format!("  {command}"));
            }
            None => ui.say(&format!(
                "The nvmimic program stays at {}, because `nvmimic self install` didn't put it there.",
                binary.display()
            )),
        }
    }
    Ok(())
}

/// Copies the running program to `target` without ever writing over an old
/// file in place. macOS remembers a binary's signature per file and stops a
/// program whose bytes change under it, so a new copy is written beside it
/// and renamed into place.
fn install_binary(binary: &Path, target: &Path) -> Result<(), String> {
    if let Some(dir) = target.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let temporary = target.with_file_name(".nvmimic.new");
    fs::copy(binary, &temporary).map_err(|e| format!("{}: {e}", temporary.display()))?;
    set_mode(&temporary, 0o755)?;
    fs::rename(&temporary, target).map_err(|e| format!("{}: {e}", target.display()))
}

/// Moves a file, also from one disk to another, where a plain rename fails.
pub(crate) fn move_file(from: &Path, to: &Path) -> Result<(), String> {
    match fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(error) if error.raw_os_error() == Some(libc::EXDEV) => fs::copy(from, to)
            .and_then(|_| fs::remove_file(from))
            .map_err(|e| format!("{}: {e}", from.display())),
        Err(error) => Err(format!("{}: {error}", from.display())),
    }
}

fn on_path(env: &Env, dir: &Path) -> bool {
    env.path
        .as_deref()
        .is_some_and(|path| std::env::split_paths(path).any(|entry| entry == dir))
}

/// Writes a file beside `path` and renames it into place. Whatever stops
/// the program, `path` holds the old contents or the new, never part of
/// each. A link at `path` is replaced, not written through.
pub(crate) fn write_file(path: &Path, text: &str, mode: u32) -> Result<(), String> {
    let temporary = unfinished(path);
    fs::write(&temporary, text).map_err(|e| format!("{}: {e}", temporary.display()))?;
    set_mode(&temporary, mode)?;
    fs::rename(&temporary, path).map_err(|e| format!("{}: {e}", path.display()))
}

pub(crate) fn set_mode(path: &Path, mode: u32) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn full_backup_path(nvm_dir: &Path) -> PathBuf {
    let name = nvm_dir
        .file_name()
        .map_or("nvm".into(), |n| n.to_string_lossy().into_owned());
    let mut n = 1;
    loop {
        let candidate = nvm_dir.with_file_name(format!("{name}.backup-{n}"));
        if !present(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    // A copy-on-write clone where the filesystem supports it.
    let flag = if cfg!(target_os = "macos") {
        "-cRp"
    } else {
        "-Rp"
    };
    let status = Command::new("cp").arg(flag).arg(from).arg(to).status();
    match status {
        Ok(status) if status.success() => Ok(()),
        _ => Err(format!(
            "copying {} to {} failed, so nothing else was changed.",
            from.display(),
            to.display()
        )),
    }
}

/// Shell startup files that contain `needle`.
fn startup_files_mentioning(env: &Env, needle: &str) -> Vec<PathBuf> {
    [
        ".bashrc",
        ".bash_profile",
        ".profile",
        ".zshrc",
        ".zprofile",
        ".zshenv",
    ]
    .iter()
    .map(|name| env.home.join(name))
    .filter(|path| fs::read_to_string(path).is_ok_and(|text| text.contains(needle)))
    .collect()
}

/// Asks on the terminal. Without one, a question cannot be answered.
pub struct TerminalUi;

impl Ui for TerminalUi {
    fn say(&mut self, line: &str) {
        println!("{line}");
    }

    fn ask(&mut self, question: &str, default: bool) -> bool {
        use std::io::{BufRead, Write};
        print!("{question} {} ", choices(default));
        let _ = io::stdout().flush();
        let mut answer = String::new();
        if io::stdin().lock().read_line(&mut answer).unwrap_or(0) == 0 {
            println!();
            return false;
        }
        match answer.trim() {
            "" => default,
            answer => matches!(answer, "y" | "Y" | "yes" | "Yes"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders_are_replaced_and_quoted() {
        let script = format!("command {BINARY_PLACEHOLDER} x; c={CACHE_PLACEHOLDER}");
        let bound = Bound {
            binary: Path::new("/a b/bin/nvmimic"),
            cache: None,
        };
        assert_eq!(bind(&script, &bound), "command '/a b/bin/nvmimic' x; c=''");
    }

    #[test]
    fn every_placeholder_is_in_the_scripts() {
        assert!(NVM_SH.contains(BINARY_PLACEHOLDER));
        assert!(!NVM_SH.contains(CACHE_PLACEHOLDER));
        for placeholder in [BINARY_PLACEHOLDER, CACHE_PLACEHOLDER, VERSION_PLACEHOLDER] {
            assert!(INIT.contains(placeholder), "{placeholder}");
        }
        let (functions, load) = INIT.split_once(LOAD_MARKER).unwrap();
        assert!(functions.contains("__nvmimic_handoff() {"));
        assert!(load.ends_with(crate::init::SECOND_START));
    }

    #[test]
    fn a_direct_init_names_the_folder_in_place_of_nvms_own_search() {
        let bound = Bound {
            binary: Path::new("/bin/nvmimic"),
            cache: None,
        };
        let sourced = init_text(&bound, None, None);
        assert!(sourced.contains("$(dirname \"${NVM_SCRIPT_SOURCE:-$0}\")"));

        let direct = init_text(&bound, Some(Path::new("/h/my nvm")), None);
        assert!(!direct.contains("NVM_SCRIPT_SOURCE:-$0"), "{direct}");
        assert!(
            direct.contains(
                "if [ -z \"${NVM_DIR-}\" ]; then\n  NVM_DIR='/h/my nvm'\n  export NVM_DIR\nelse\n"
            ),
            "{direct}"
        );
    }

    #[test]
    fn manifest_round_trips() {
        let manifest = Manifest {
            version: "0.1.0".into(),
            nvm_dir: Some("/h/.nvm".into()),
            binary: Some("/h/.local/bin/nvmimic".into()),
            placed: true,
            data: Some("/h/.local/share/nvmimic".into()),
            cache: Some("/h/.cache/nvmimic".into()),
            backed_up: vec!["nvm.sh".into()],
            links: [(
                "nvm.sh".to_string(),
                PathBuf::from("/brew/opt/my nvm/nvm.sh"),
            )]
            .into(),
            created: vec!["bash_completion".into()],
            fingerprints: [("nvm.sh".to_string(), "ab12".to_string())].into(),
            ejected: true,
            previous: Some("0.0.9".into()),
        };
        assert_eq!(Manifest::parse(&manifest.render()), manifest);
    }

    /// A `Ui` that keeps what it was told and gives one answer.
    struct Recorded {
        said: Vec<String>,
        asked: Vec<String>,
        answer: bool,
    }

    impl Ui for Recorded {
        fn say(&mut self, line: &str) {
            self.said.push(line.to_string());
        }

        fn ask(&mut self, question: &str, _default: bool) -> bool {
            self.asked.push(question.to_string());
            self.answer
        }
    }

    /// A throwaway home with nvm's three scripts in `~/.nvm`, each holding
    /// its own name.
    fn home_with_nvm(label: &str) -> Env {
        let root = std::env::temp_dir().join(format!("nvmimic-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".nvm")).unwrap();
        for name in SCRIPTS {
            fs::write(root.join(".nvm").join(name), format!("nvm's own {name}\n")).unwrap();
        }
        fs::write(root.join("program"), "a stand-in for the nvmimic program\n").unwrap();
        let home = root.to_string_lossy().into_owned();
        Env {
            home: root.clone(),
            nvm_dir: None,
            xdg_config_home: None,
            homebrew_prefix: Some(format!("{home}/no-homebrew")),
            cargo_home: None,
            path: None,
            dirs: Dirs::resolve(|name| (name == "HOME").then(|| home.clone())),
            install_path_given: false,
        }
    }

    fn quiet() -> Recorded {
        Recorded {
            said: Vec::new(),
            asked: Vec::new(),
            answer: true,
        }
    }

    /// Whether `$NVM_DIR` holds nvm's own three scripts again.
    fn nvm_is_back(env: &Env) -> bool {
        SCRIPTS.iter().all(|name| {
            fs::read_to_string(env.nvm_dir().join(name))
                .is_ok_and(|text| text == format!("nvm's own {name}\n"))
        })
    }

    #[test]
    fn an_install_stopped_at_any_step_loses_none_of_nvms_scripts() {
        let force = Options {
            force: true,
            ..Options::default()
        };
        let mut stopped_at = 0;
        loop {
            // Stop the install before step `stopped_at`, then do what
            // someone would do next: remove nvmimic, or install it again
            // and remove it later.
            let mut finished = false;
            for install_again in [false, true] {
                let env = home_with_nvm("stopped");
                let program = env.home.join("program");
                STEPS.set(0);
                STOP_BEFORE.set(Some(stopped_at));
                let result = self_install(&env, &program, &force, &mut quiet());
                STOP_BEFORE.set(None);
                finished = result.is_ok();
                if finished {
                    break;
                }
                assert_eq!(result, Err("interrupted".to_string()));

                if install_again {
                    self_install(&env, &program, &force, &mut quiet())
                        .unwrap_or_else(|e| panic!("step {stopped_at}, second install: {e}"));
                }
                let installed = Manifest::read(env.dirs().unwrap()).is_some();
                assert!(installed || !install_again);
                if installed {
                    self_uninstall(&env, &force, &mut quiet())
                        .unwrap_or_else(|e| panic!("step {stopped_at}, uninstall: {e}"));
                }
                assert!(
                    nvm_is_back(&env),
                    "stopped before step {stopped_at} (install again: {install_again}): nvm's \
                     scripts did not come back"
                );
            }
            if finished {
                break;
            }
            stopped_at += 1;
        }
        // Three scripts to keep, a manifest, a program and three scripts
        // to write: an install has at least that many steps.
        assert!(stopped_at >= 8, "only {stopped_at} steps");
    }

    #[test]
    fn a_purge_at_a_terminal_shows_the_folder_and_its_size_before_it_asks() {
        let root = std::env::temp_dir().join(format!("nvmimic-purge-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let home = root.to_string_lossy().into_owned();
        let env = Env {
            home: root.clone(),
            nvm_dir: None,
            xdg_config_home: None,
            homebrew_prefix: Some(format!("{home}/no-homebrew")),
            cargo_home: None,
            path: None,
            dirs: Dirs::resolve(|name| (name == "HOME").then(|| home.clone())),
            install_path_given: false,
        };
        let dirs = env.dirs().unwrap();
        let nvm_dir = root.join(".nvm");
        fs::create_dir_all(nvm_dir.join("versions/node/v20.1.0")).unwrap();
        fs::write(nvm_dir.join("versions/node/v20.1.0/node"), vec![0; 3000]).unwrap();
        let manifest = Manifest {
            nvm_dir: Some(nvm_dir.clone()),
            binary: Some(root.join("elsewhere/nvmimic")),
            ..Manifest::default()
        };
        fs::create_dir_all(&dirs.state).unwrap();
        fs::write(dirs.manifest(), manifest.render()).unwrap();
        let options = Options {
            purge: true,
            terminal: true,
            ..Options::default()
        };

        let mut ui = Recorded {
            said: Vec::new(),
            asked: Vec::new(),
            answer: false,
        };
        let stopped = self_uninstall(&env, &options, &mut ui);
        assert_eq!(stopped, Err("stopped. Nothing was changed.".to_string()));
        assert_eq!(
            ui.said[0],
            format!(
                "This will delete {} and everything in it, your Node versions and aliases included (3 KB).",
                nvm_dir.display()
            )
        );
        assert_eq!(
            ui.asked,
            [format!("Delete {} for good?", nvm_dir.display())]
        );
        assert!(nvm_dir.join("versions/node/v20.1.0/node").is_file());

        ui.answer = true;
        self_uninstall(&env, &options, &mut ui).unwrap();
        assert!(!nvm_dir.exists());
        assert!(!dirs.manifest().exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_purge_never_takes_the_home_folder() {
        let env = Env {
            home: PathBuf::from("/nowhere/home/me"),
            nvm_dir: None,
            xdg_config_home: None,
            homebrew_prefix: None,
            cargo_home: None,
            path: None,
            dirs: None,
            install_path_given: false,
        };
        for dir in ["/", "/nowhere/home", "/nowhere/home/me", "relative/.nvm"] {
            assert!(purge_allowed(Path::new(dir), &env).is_err(), "{dir}");
        }
        assert!(purge_allowed(Path::new("/nowhere/home/me/.nvm"), &env).is_ok());
    }

    #[test]
    fn sizes_read_the_way_people_say_them() {
        assert_eq!(human_size(512), "512 bytes");
        assert_eq!(human_size(3_000), "3 KB");
        assert_eq!(human_size(48_400_000), "48 MB");
        assert_eq!(human_size(1_260_000_000), "1.3 GB");
    }

    #[test]
    fn installed_scripts_carry_the_marker() {
        assert!(NVM_SH.contains(OWN_MARKER));
    }
}
