//! `nvmimic self update` and `nvmimic self rollback`.
//!
//! An update never writes over the installed program. macOS remembers a
//! program's signature per file and stops one whose bytes change under it.
//! So the new program is downloaded next to the old one, checked, and then
//! swapped in by renaming. The old one is kept in nvmimic's state folder,
//! and `self rollback` swaps it back.
//!
//! The checks, in order: the download's checksum, who signed it, and that
//! it runs and reports the version that was promised. After the swap the
//! new program writes its own `nvm.sh` by running its `self install`. If
//! that fails, the old program comes back by itself.

use crate::dirs::Dirs;
use crate::identity::Identity;
use crate::install::{Env, Managed, Manifest, Options, Ui, ask, managed_by, move_file};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Where releases are published.
pub const RELEASES: &str = "https://github.com/tsilenzio/nvmimic/releases";

/// The Apple team that signs releases. Only a release build has it, and
/// `self update` accepts only a download signed by the same team.
pub const TEAM_ID: Option<&str> = option_env!("NVMIMIC_TEAM_ID");

/// The release file for this kind of machine, if releases are made for it.
pub fn archive_name() -> Option<&'static str> {
    cfg!(target_os = "macos").then_some("nvmimic-darwin-universal.tar.gz")
}

/// [`archive_name`] for a program built on this library: the same name
/// with the program's own in it.
pub fn archive_name_for(identity: &Identity) -> Option<String> {
    archive_name().map(|name| name.replacen("nvmimic", identity.name, 1))
}

/// Where the program `self update` set aside is kept.
pub fn previous_path(dirs: &Dirs, identity: &Identity, version: &str) -> PathBuf {
    dirs.state
        .join("previous")
        .join(format!("{}-{version}", identity.name))
}

/// What the updater needs to know besides the installation itself.
#[derive(Clone, Debug)]
pub struct Source {
    /// The address the newest release's files are under.
    pub latest: String,
    /// Accept a download without checking who signed it. For a program
    /// that was not built as a release, which has no team to check against.
    pub allow_unsigned: bool,
}

impl Source {
    /// nvmimic's releases, or the address `NVMIMIC_RELEASES_URL` names.
    pub fn from_process() -> Source {
        Source::from_process_for(&Identity::NVMIMIC).expect("nvmimic has releases")
    }

    /// [`Source::from_process`] for another program built on this library.
    /// `None` when that program has no releases of its own.
    pub fn from_process_for(identity: &Identity) -> Option<Source> {
        let var = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
        let release = identity.release.as_ref()?;
        Some(Source {
            latest: var(&identity.setting("RELEASES_URL"))
                .unwrap_or_else(|| format!("{}/latest/download", release.url)),
            allow_unsigned: var(&identity.setting("UPDATE_UNSIGNED")).is_some(),
        })
    }
}

/// The installation to work on: its manifest and where its program is.
fn installed(env: &Env) -> Result<(&Dirs, Manifest, PathBuf), String> {
    let app = env.identity.name;
    let dirs = env.dirs()?;
    let manifest = Manifest::read(dirs).ok_or(format!(
        "{app} is not installed yet. Run `{app} self install` first."
    ))?;
    let binary = manifest.binary.clone().ok_or(format!(
        "the record of the install does not say where the {app} program is."
    ))?;
    Ok((dirs, manifest, binary))
}

fn fetch(url: &str, to: &Path) -> Result<(), String> {
    let status = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--output",
        ])
        .arg(to)
        .arg(url)
        .status()
        .map_err(|e| format!("curl: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("could not download {url}"))
    }
}

/// What a program prints for `--version`.
fn version_of(program: &Path) -> Option<String> {
    let output = Command::new(program).arg("--version").output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Checks that Apple's signature on `program` is whole and was made by
/// `team`.
fn signed_by(program: &Path, team: &str) -> bool {
    let requirement =
        format!("=anchor apple generic and certificate leaf[subject.OU] = \"{team}\"");
    Command::new("/usr/bin/codesign")
        .args(["--verify", "--strict", "-R"])
        .arg(requirement)
        .arg(program)
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Lets the program at `binary` write its own scripts into `$NVM_DIR`.
fn write_scripts(identity: &Identity, binary: &Path, manifest: &Manifest) -> bool {
    let mut command = Command::new(binary);
    // The program stays where it is, whatever the environment says now.
    command
        .args(["self", "install", "--force"])
        .env(identity.setting("INSTALL_PATH"), binary);
    if let Some(nvm_dir) = &manifest.nvm_dir {
        command.env("NVM_DIR", nvm_dir);
    }
    command.output().is_ok_and(|output| output.status.success())
}

/// Puts `new` where `binary` is and keeps the program that was there as
/// the previous one. Leaves everything as it was when the new program
/// cannot set itself up.
fn swap(
    identity: &Identity,
    dirs: &Dirs,
    manifest: &mut Manifest,
    binary: &Path,
    new: &Path,
    new_version: &str,
) -> Result<(), String> {
    let old_version = manifest.version.clone();
    let kept = previous_path(dirs, identity, &old_version);
    if let Some(dir) = kept.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let before = manifest.clone();
    move_file(binary, &kept)?;
    fs::rename(new, binary).map_err(|e| format!("{}: {e}", binary.display()))?;
    manifest.version = new_version.to_string();
    manifest.previous = Some(old_version.clone());
    manifest.write(dirs, identity)?;

    if write_scripts(identity, binary, manifest) {
        // Only one earlier program is kept.
        if let Some(older) = before.previous.filter(|older| *older != old_version) {
            let _ = fs::remove_file(previous_path(dirs, identity, &older));
        }
        return Ok(());
    }
    let _ = fs::remove_file(binary);
    move_file(&kept, binary)?;
    *manifest = before;
    manifest.write(dirs, identity)?;
    write_scripts(identity, binary, manifest);
    let app = identity.name;
    Err(format!(
        "{app} {new_version} could not set itself up, so {app} {old_version} was put back."
    ))
}

/// `nvmimic self update`.
pub fn self_update(
    env: &Env,
    source: &Source,
    options: &Options,
    ui: &mut dyn Ui,
) -> Result<(), String> {
    let (dirs, mut manifest, binary) = installed(env)?;
    let identity = env.identity;
    let app = identity.name;
    match managed_by(&binary, env) {
        Some(Managed::Homebrew) => {
            ui.say(&format!(
                "Homebrew installed this {app}, so Homebrew updates it:"
            ));
            ui.say(&format!("  brew upgrade {}", identity.packages.homebrew));
            return Ok(());
        }
        Some(Managed::Cargo) => {
            ui.say(&format!("cargo installed this {app}, so cargo updates it:"));
            ui.say(&format!("  cargo install {}", identity.packages.cargo));
            return Ok(());
        }
        None => {}
    }
    let no_releases = || format!("there are no {app} releases for this system yet.");
    let release = identity.release.as_ref().ok_or_else(no_releases)?;
    let archive = archive_name_for(identity).ok_or_else(no_releases)?;
    if release.team_id.is_none() && cfg!(target_os = "macos") && !source.allow_unsigned {
        return Err(format!(
            "this copy of {app} was not built as a release, so it cannot check who signed a \
             download. Install a released {app} to get updates."
        ));
    }

    let folder = binary
        .parent()
        .ok_or(format!("the {app} program has no folder."))?;
    let work = folder.join(format!(".{app}-update-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).map_err(|e| format!("{}: {e}", work.display()))?;
    let result = update_in(
        &work,
        identity,
        release.team_id,
        dirs,
        &mut manifest,
        &binary,
        &archive,
        source,
        options,
        ui,
    );
    let _ = fs::remove_dir_all(&work);
    result
}

#[allow(clippy::too_many_arguments)]
fn update_in(
    work: &Path,
    identity: &Identity,
    team_id: Option<&str>,
    dirs: &Dirs,
    manifest: &mut Manifest,
    binary: &Path,
    archive: &str,
    source: &Source,
    options: &Options,
    ui: &mut dyn Ui,
) -> Result<(), String> {
    let app = identity.name;
    let version_file = work.join("VERSION");
    fetch(&format!("{}/VERSION", source.latest), &version_file)?;
    let newest = fs::read_to_string(&version_file)
        .unwrap_or_default()
        .trim()
        .to_string();
    if newest.is_empty() || newest.contains(char::is_whitespace) {
        return Err("the release does not say which version it is.".to_string());
    }
    if newest == manifest.version {
        ui.say(&format!("{app} {newest} is the newest version."));
        return Ok(());
    }
    if !ask(
        ui,
        options,
        &format!("Update {app} from {} to {newest}?", manifest.version),
        true,
    ) {
        return Err("stopped. Nothing was changed.".to_string());
    }
    if options.dry_run {
        ui.say(&format!(
            "Would download {app} {newest}, check it, and put it at {}.",
            binary.display()
        ));
        return Ok(());
    }

    let download = work.join(archive);
    let sums = work.join("SHA256SUMS");
    fetch(&format!("{}/{archive}", source.latest), &download)?;
    fetch(&format!("{}/SHA256SUMS", source.latest), &sums)?;
    let bytes = fs::read(&download).map_err(|e| format!("{}: {e}", download.display()))?;
    let sum: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let listed = fs::read_to_string(&sums).unwrap_or_default();
    if !listed
        .lines()
        .any(|line| line.split_whitespace().eq([sum.as_str(), archive]))
    {
        return Err("the download does not match its checksum. Nothing was changed.".to_string());
    }

    let unpacked = Command::new("tar")
        .arg("-xzf")
        .arg(&download)
        .arg("-C")
        .arg(work)
        .status();
    let new = work.join(app).join(app);
    if !unpacked.is_ok_and(|status| status.success()) || !new.is_file() {
        return Err("the download could not be unpacked. Nothing was changed.".to_string());
    }
    match team_id {
        Some(team) if cfg!(target_os = "macos") => {
            if !signed_by(&new, team) {
                return Err(format!(
                    "the download is not signed by {app}'s developer. Nothing was changed."
                ));
            }
        }
        _ => ui.say("Not checking who signed the download."),
    }
    if version_of(&new).as_deref() != Some(&newest) {
        return Err(format!(
            "the download does not run as {app} {newest}. Nothing was changed."
        ));
    }

    let old = manifest.version.clone();
    swap(identity, dirs, manifest, binary, &new, &newest)?;
    ui.say(&format!(
        "Updated {app} from {old} to {newest}. `{app} self rollback` goes back to {old}."
    ));
    ui.say("Open a new shell to use it.");
    Ok(())
}

/// `nvmimic self rollback`.
pub fn self_rollback(env: &Env, options: &Options, ui: &mut dyn Ui) -> Result<(), String> {
    let (dirs, mut manifest, binary) = installed(env)?;
    let identity = env.identity;
    let app = identity.name;
    let Some(previous) = manifest.previous.clone() else {
        return Err(format!("there is no earlier {app} to go back to."));
    };
    let kept = previous_path(dirs, identity, &previous);
    if !kept.is_file() {
        return Err(format!("{} is missing.", kept.display()));
    }
    if !ask(
        ui,
        options,
        &format!("Go back from {app} {} to {previous}?", manifest.version),
        true,
    ) {
        return Err("stopped. Nothing was changed.".to_string());
    }
    if options.dry_run {
        ui.say(&format!(
            "Would put {app} {previous} back at {}.",
            binary.display()
        ));
        return Ok(());
    }
    // The earlier program goes beside the current one first, so the swap
    // itself is a rename.
    let beside = binary.with_file_name(".nvmimic.rollback");
    fs::copy(&kept, &beside).map_err(|e| format!("{}: {e}", beside.display()))?;
    crate::install::set_mode(&beside, 0o755)?;
    let current = manifest.version.clone();
    swap(identity, dirs, &mut manifest, &binary, &beside, &previous)?;
    ui.say(&format!(
        "{app} {previous} is back. `{app} self rollback` again returns to {current}."
    ));
    Ok(())
}
