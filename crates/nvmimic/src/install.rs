//! Installing nvmimic into `$NVM_DIR`, taking over an nvm that is already
//! there, and putting that nvm back.
//!
//! Only nvm's three scripts are replaced. `versions/`, `alias/` and `.cache/`
//! are never touched, so installed Node versions keep working with either.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// How `shell/nvm.sh` names the binary before it is installed.
pub const BINARY_PLACEHOLDER: &str = "\"${NVM_DIR}/.nvmimic/bin/nvm\"";

const NVM_SH: &str = include_str!("../../../shell/nvm.sh");
const NVM_EXEC: &str = include_str!("../../../shell/nvm-exec");
const BASH_COMPLETION: &str = include_str!("../../../shell/bash_completion");

/// The scripts nvm keeps in `$NVM_DIR`, which nvmimic replaces.
const SCRIPTS: &[&str] = &["nvm.sh", "nvm-exec", "bash_completion"];

/// Text only nvmimic's own `nvm.sh` contains.
const OWN_MARKER: &str = "as nvmimic provides it";

/// The `nvm.sh` to install, pointing at the binary by absolute path. nvm
/// lets `$NVM_DIR` change after sourcing, and the script itself works out
/// where it lives, so the binary cannot be found from `$NVM_DIR` alone.
pub fn bind_binary(nvm_sh: &str, nvm_link: &str) -> String {
    nvm_sh.replace(BINARY_PLACEHOLDER, &crate::shell::quote(nvm_link))
}

/// What the installer talks to: lines to show, and yes-or-no questions.
pub trait Ui {
    fn say(&mut self, line: &str);
    /// Asks a question whose default answer is no.
    fn ask(&mut self, question: &str) -> bool;
}

#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Answer yes to every question.
    pub yes: bool,
    /// Say what would happen and change nothing.
    pub dry_run: bool,
    /// Also put a `nvmimic` link in `~/.local/bin`.
    pub link: bool,
}

/// The parts of the environment the installer looks at.
#[derive(Clone, Debug)]
pub struct Env {
    pub home: PathBuf,
    pub nvm_dir: Option<String>,
    pub xdg_config_home: Option<String>,
    pub homebrew_prefix: Option<String>,
}

impl Env {
    pub fn from_process() -> Env {
        let var = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
        Env {
            home: var("HOME").map(PathBuf::from).unwrap_or_default(),
            nvm_dir: var("NVM_DIR"),
            xdg_config_home: var("XDG_CONFIG_HOME"),
            homebrew_prefix: var("HOMEBREW_PREFIX"),
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

/// The record of what an install changed, kept at `.nvmimic/manifest`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Manifest {
    pub version: String,
    /// Scripts that were nvm's and now sit in `.nvmimic/backup/`.
    pub backed_up: Vec<String>,
    /// Scripts nvmimic added where there were none.
    pub created: Vec<String>,
    /// A `nvmimic` link outside `$NVM_DIR`, if one was made.
    pub link: Option<PathBuf>,
}

impl Manifest {
    fn render(&self) -> String {
        let mut text = String::from(
            "# What `nvmimic self install` changed. `nvmimic self uninstall` reads it.\n",
        );
        text.push_str(&format!("version {}\n", self.version));
        for name in &self.backed_up {
            text.push_str(&format!("backup {name}\n"));
        }
        for name in &self.created {
            text.push_str(&format!("created {name}\n"));
        }
        if let Some(link) = &self.link {
            text.push_str(&format!("link {}\n", link.display()));
        }
        text
    }

    fn parse(text: &str) -> Manifest {
        let mut manifest = Manifest::default();
        for line in text.lines().filter(|line| !line.starts_with('#')) {
            match line.split_once(' ') {
                Some(("version", value)) => manifest.version = value.to_string(),
                Some(("backup", value)) => manifest.backed_up.push(value.to_string()),
                Some(("created", value)) => manifest.created.push(value.to_string()),
                Some(("link", value)) => manifest.link = Some(PathBuf::from(value)),
                _ => {}
            }
        }
        manifest
    }
}

fn own_dir(nvm_dir: &Path) -> PathBuf {
    nvm_dir.join(".nvmimic")
}

fn is_ours(path: &Path) -> bool {
    fs::read_to_string(path).is_ok_and(|text| text.contains(OWN_MARKER))
}

/// Whether a script is present, a dangling symlink included.
fn present(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn ask(ui: &mut dyn Ui, options: &Options, question: &str) -> bool {
    if options.yes {
        ui.say(&format!("{question} [y/N] y (--yes)"));
        true
    } else {
        ui.ask(question)
    }
}

/// `nvmimic self install`.
pub fn self_install(
    env: &Env,
    binary: &Path,
    options: &Options,
    ui: &mut dyn Ui,
) -> Result<(), String> {
    let nvm_dir = env.nvm_dir();
    let own = own_dir(&nvm_dir);
    let manifest_path = own.join("manifest");
    let reinstall = manifest_path.is_file();

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
        if !ask(ui, options, "Run `brew uninstall nvm` now?") {
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

    let existing: Vec<&str> = SCRIPTS
        .iter()
        .copied()
        .filter(|name| present(&nvm_dir.join(name)) && !is_ours(&nvm_dir.join(name)))
        .collect();
    if !existing.is_empty() && !reinstall {
        ui.say(&format!("Found nvm in {}.", nvm_dir.display()));
        ui.say("nvmimic aims to be a drop-in replacement: it uses the Node versions, aliases");
        ui.say("and settings that are already there, and only swaps out nvm's scripts.");
        ui.say(&format!(
            "The scripts it replaces ({}) are kept in {} and come back with `nvmimic self uninstall`.",
            existing.join(", "),
            own.join("backup").display()
        ));
        if nvm_dir.join(".git").exists() {
            ui.say("This nvm was installed with git. While nvmimic is installed, `git status` in that");
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
        if !ask(ui, options, "Install nvmimic now?") {
            return Err("stopped. Nothing was changed.".to_string());
        }
    }

    let mut manifest = if reinstall {
        Manifest::parse(&fs::read_to_string(&manifest_path).unwrap_or_default())
    } else {
        Manifest::default()
    };
    manifest.version = env!("CARGO_PKG_VERSION").to_string();

    let bin = own.join("bin");
    let link = bin.join("nvm");
    if options.dry_run {
        ui.say(&format!(
            "Would install nvmimic into {}.",
            nvm_dir.display()
        ));
        for name in &existing {
            ui.say(&format!(
                "Would move {} to {}.",
                nvm_dir.join(name).display(),
                own.join("backup").join(name).display()
            ));
        }
        ui.say(&format!(
            "Would copy this program to {}.",
            bin.join("nvmimic").display()
        ));
        return Ok(());
    }

    for dir in [&bin, &own.join("backup")] {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    for name in SCRIPTS {
        let path = nvm_dir.join(name);
        if existing.contains(name) {
            let backup = own.join("backup").join(name);
            fs::rename(&path, &backup).map_err(|e| format!("{}: {e}", path.display()))?;
            if !manifest.backed_up.iter().any(|n| n == name) {
                manifest.backed_up.push(name.to_string());
            }
        } else if !present(&path) && !manifest.created.iter().any(|n| n == name) {
            manifest.created.push(name.to_string());
        }
    }

    ui.say(&format!("Installing nvmimic into {}.", nvm_dir.display()));
    install_binary(binary, &bin)?;
    let _ = fs::remove_file(&link);
    std::os::unix::fs::symlink("nvmimic", &link).map_err(|e| format!("{}: {e}", link.display()))?;

    let link_text = link.display().to_string();
    write_file(
        &nvm_dir.join("nvm.sh"),
        &bind_binary(NVM_SH, &link_text),
        0o644,
    )?;
    write_file(&nvm_dir.join("nvm-exec"), NVM_EXEC, 0o755)?;
    write_file(&nvm_dir.join("bash_completion"), BASH_COMPLETION, 0o644)?;

    if options.link || (!options.yes && ui.ask("Also add a `nvmimic` command to ~/.local/bin?")) {
        let local_bin = env.home.join(".local/bin");
        fs::create_dir_all(&local_bin).map_err(|e| format!("{}: {e}", local_bin.display()))?;
        let target = local_bin.join("nvmimic");
        let _ = fs::remove_file(&target);
        std::os::unix::fs::symlink(bin.join("nvmimic"), &target)
            .map_err(|e| format!("{}: {e}", target.display()))?;
        manifest.link = Some(target);
    }

    // In a git-clone install, this keeps nvmimic's own folder out of
    // `git status`. The swapped scripts still show as modified there.
    write_file(&own.join(".gitignore"), "*\n", 0o644)?;

    write_file(&manifest_path, &manifest.render(), 0o644)?;
    ui.say("Done.");
    if startup_files_mentioning(env, "/nvm.sh").is_empty() {
        ui.say("To load nvm in new shells, add these lines to your shell's startup file:");
        ui.say(&format!("  export NVM_DIR=\"{}\"", nvm_dir.display()));
        ui.say("  [ -s \"$NVM_DIR/nvm.sh\" ] && \\. \"$NVM_DIR/nvm.sh\"  # This loads nvm");
    } else {
        ui.say("Open a new shell to start using it.");
    }
    Ok(())
}

/// `nvmimic self uninstall`.
pub fn self_uninstall(env: &Env, options: &Options, ui: &mut dyn Ui) -> Result<(), String> {
    let nvm_dir = env.nvm_dir();
    let own = own_dir(&nvm_dir);
    let Ok(text) = fs::read_to_string(own.join("manifest")) else {
        return Err(format!(
            "nvmimic is not installed in {}.",
            nvm_dir.display()
        ));
    };
    let manifest = Manifest::parse(&text);
    if !manifest.backed_up.is_empty() {
        ui.say(&format!(
            "This puts nvm's own {} back.",
            manifest.backed_up.join(", ")
        ));
    }
    ui.say("Your Node versions and aliases stay as they are.");
    if !ask(
        ui,
        options,
        &format!("Remove nvmimic from {}?", nvm_dir.display()),
    ) {
        return Err("stopped. Nothing was changed.".to_string());
    }
    if options.dry_run {
        ui.say(&format!("Would remove {}.", own.display()));
        return Ok(());
    }
    for name in &manifest.backed_up {
        let backup = own.join("backup").join(name);
        fs::rename(&backup, nvm_dir.join(name))
            .map_err(|e| format!("{}: {e}", backup.display()))?;
    }
    for name in &manifest.created {
        let _ = fs::remove_file(nvm_dir.join(name));
    }
    if let Some(link) = &manifest.link
        && fs::read_link(link).is_ok_and(|target| target.starts_with(&own))
    {
        let _ = fs::remove_file(link);
    }
    fs::remove_dir_all(&own).map_err(|e| format!("{}: {e}", own.display()))?;
    ui.say("Done. nvmimic is gone.");
    Ok(())
}

/// Copies the running program into `bin/nvmimic` without ever writing over
/// the old file in place. macOS remembers a binary's signature per file and
/// stops a program whose bytes change under it, so a new copy is written
/// beside it and renamed into place.
fn install_binary(binary: &Path, bin: &Path) -> Result<(), String> {
    let target = bin.join("nvmimic");
    if fs::canonicalize(binary).ok() == fs::canonicalize(&target).ok() && target.exists() {
        return Ok(());
    }
    let temporary = bin.join(".nvmimic.new");
    fs::copy(binary, &temporary).map_err(|e| format!("{}: {e}", temporary.display()))?;
    set_mode(&temporary, 0o755)?;
    fs::rename(&temporary, &target).map_err(|e| format!("{}: {e}", target.display()))
}

fn write_file(path: &Path, text: &str, mode: u32) -> Result<(), String> {
    fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    set_mode(path, mode)
}

fn set_mode(path: &Path, mode: u32) -> Result<(), String> {
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

    fn ask(&mut self, question: &str) -> bool {
        use std::io::{BufRead, Write};
        print!("{question} [y/N] ");
        let _ = io::stdout().flush();
        let mut answer = String::new();
        if io::stdin().lock().read_line(&mut answer).unwrap_or(0) == 0 {
            println!();
            return false;
        }
        matches!(answer.trim(), "y" | "Y" | "yes" | "Yes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_is_replaced_and_quoted() {
        let script = format!("command {BINARY_PLACEHOLDER} __fn");
        assert_eq!(
            bind_binary(&script, "/a b/.nvmimic/bin/nvm"),
            "command '/a b/.nvmimic/bin/nvm' __fn"
        );
    }

    #[test]
    fn manifest_round_trips() {
        let manifest = Manifest {
            version: "0.1.0".into(),
            backed_up: vec!["nvm.sh".into()],
            created: vec!["bash_completion".into()],
            link: Some("/h/.local/bin/nvmimic".into()),
        };
        assert_eq!(Manifest::parse(&manifest.render()), manifest);
    }

    #[test]
    fn installed_scripts_carry_the_marker() {
        assert!(NVM_SH.contains(OWN_MARKER));
    }
}
