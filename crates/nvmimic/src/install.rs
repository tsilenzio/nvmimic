//! Installing nvmimic, taking over an nvm that is already in `$NVM_DIR`, and
//! putting that nvm back.
//!
//! In `$NVM_DIR` only nvm's three scripts are replaced. `versions/`, `alias/`
//! and `.cache/` are never touched, so installed Node versions keep working
//! with either. Everything else nvmimic needs lives outside `$NVM_DIR`, see
//! [`crate::dirs`].

use crate::dirs::{CACHE_KINDS, Dirs};
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

/// What `nvmimic init` prints: the shell functions, then the lines that
/// load the default version.
pub fn init_text(paths: &Bound) -> String {
    bind(INIT, paths)
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
    /// Answer yes to every question.
    pub yes: bool,
    /// Say what would happen and change nothing.
    pub dry_run: bool,
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
    /// Scripts that were nvm's and now wait in the backup folder.
    pub backed_up: Vec<String>,
    /// Scripts nvmimic added where there were none.
    pub created: Vec<String>,
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
        for name in &self.created {
            text.push_str(&format!("created {name}\n"));
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
                "created" => manifest.created.push(value.to_string()),
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
    if options.yes {
        ui.say(&format!("{question} {} y (--yes)", choices(default)));
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

    if let Some(prefix) = env.homebrew_nvm() {
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
        } else {
            let brew = prefix.join("bin/brew");
            let brew = if brew.is_file() {
                brew
            } else {
                PathBuf::from("brew")
            };
            let status = Command::new(&brew).args(["uninstall", "nvm"]).status();
            if !status.is_ok_and(|status| status.success()) {
                return Err(
                    "`brew uninstall nvm` did not finish, so nothing was changed.".to_string(),
                );
            }
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

    if !existing.is_empty() && reinstall {
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
                "Would move {} to {}.",
                nvm_dir.join(name).display(),
                backup.join(name).display()
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

    for dir in [&nvm_dir, &dirs.data, &backup, &dirs.cache] {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    for name in SCRIPTS {
        let path = nvm_dir.join(name);
        if existing.contains(name) {
            move_file(&path, &backup.join(name))?;
            if !manifest.backed_up.iter().any(|n| n == name) {
                manifest.backed_up.push(name.to_string());
            }
            // A script nvm has since put there is nvm's to keep on uninstall.
            manifest.created.retain(|n| n != name);
        } else if !present(&path) && !manifest.created.iter().any(|n| n == name) {
            manifest.created.push(name.to_string());
        }
    }

    ui.say(&format!("Installing nvmimic for {}.", nvm_dir.display()));
    if let BinaryPlan::Copy(target) = &plan {
        install_binary(running, target)?;
        ui.say(&format!("The nvmimic program is at {}.", target.display()));
    }

    write_file(&nvm_dir.join("nvm.sh"), &nvm_sh(plan.path()), 0o644)?;
    write_file(&nvm_dir.join("nvm-exec"), NVM_EXEC, 0o755)?;
    write_file(&nvm_dir.join("bash_completion"), BASH_COMPLETION, 0o644)?;

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
    write_file(&dirs.manifest(), &manifest.render(), 0o644)?;

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

/// `nvmimic self uninstall`.
pub fn self_uninstall(env: &Env, options: &Options, ui: &mut dyn Ui) -> Result<(), String> {
    let dirs = env.dirs()?;
    let Some(manifest) = Manifest::read(dirs) else {
        return Err(format!(
            "nvmimic is not installed. There is no record of an install in {}.",
            dirs.state.display()
        ));
    };
    let nvm_dir = manifest.nvm_dir.clone().unwrap_or_else(|| env.nvm_dir());
    let backup = dirs.backup();
    let restores = !manifest.backed_up.is_empty();
    if restores {
        ui.say(&format!(
            "This puts nvm's own {} back, so nvm works as it did before nvmimic.",
            manifest.backed_up.join(", ")
        ));
    } else {
        ui.say(&format!(
            "There was no nvm in {} before nvmimic, so afterwards there is no `nvm` command.",
            nvm_dir.display()
        ));
    }
    ui.say("Your Node versions and aliases stay as they are.");
    if !ask(
        ui,
        options,
        &format!("Remove nvmimic from {}?", nvm_dir.display()),
        false,
    ) {
        return Err("stopped. Nothing was changed.".to_string());
    }
    let data = manifest.data.as_ref().unwrap_or(&dirs.data);
    let cache = manifest.cache.as_ref().unwrap_or(&dirs.cache);
    let placed = manifest.binary.as_ref().filter(|_| manifest.placed);
    if options.dry_run {
        ui.say(&format!(
            "Would remove nvmimic's files from {}, {}, {} and {}.",
            nvm_dir.display(),
            data.display(),
            dirs.state.display(),
            cache.display()
        ));
        if let Some(binary) = placed {
            ui.say(&format!("Would remove {}.", binary.display()));
        }
        return Ok(());
    }
    for name in &manifest.backed_up {
        move_file(&backup.join(name), &nvm_dir.join(name))?;
    }
    for name in &manifest.created {
        let _ = fs::remove_file(nvm_dir.join(name));
    }

    // Only what nvmimic put into its folders is removed, file by file. A
    // folder that then still holds something of yours stays. The two files
    // in the data folder are what earlier versions kept there.
    for file in ["nvm-functions.sh", "nvmimic-functions.sh"] {
        let _ = fs::remove_file(data.join(file));
    }
    let _ = fs::remove_dir(data);
    for kind in CACHE_KINDS {
        let _ = fs::remove_dir_all(cache.join(kind));
    }
    let _ = fs::remove_dir(cache);
    if let Some(binary) = placed {
        let _ = fs::remove_file(binary);
    }
    let _ = fs::remove_file(dirs.manifest());
    if let Some(version) = &manifest.previous {
        let _ = fs::remove_file(crate::update::previous_path(dirs, version));
    }
    let _ = fs::remove_dir(dirs.state.join("previous"));
    let _ = fs::remove_dir(&backup);
    let _ = fs::remove_dir(&dirs.state);

    ui.say(if restores {
        "Done. nvmimic is gone and nvm is back."
    } else {
        "Done. nvmimic is gone."
    });
    if let Some(binary) = manifest.binary.as_ref().filter(|_| !manifest.placed) {
        ui.say(&format!(
            "The nvmimic program stays at {}, because `nvmimic self install` didn't put it there.",
            binary.display()
        ));
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

pub(crate) fn write_file(path: &Path, text: &str, mode: u32) -> Result<(), String> {
    fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    set_mode(path, mode)
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
        assert!(load.contains("nvm_process_parameters \"$@\""));
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
            created: vec!["bash_completion".into()],
            previous: Some("0.0.9".into()),
        };
        assert_eq!(Manifest::parse(&manifest.render()), manifest);
    }

    #[test]
    fn installed_scripts_carry_the_marker() {
        assert!(NVM_SH.contains(OWN_MARKER));
    }
}
