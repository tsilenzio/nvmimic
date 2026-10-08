//! `nvm use`: points `PATH` at an installed version.

use super::deactivate;
use crate::context::Nvm;
use crate::nvm::{local, npm, nvmrc, paths, version};
use crate::shell::Effect;
use crate::text::messages;

/// nvm: nvm:use
pub fn run(nvm: &mut Nvm, args: &[String]) -> i32 {
    let mut provided = String::new();
    let mut silent = false;
    let mut delete_prefix = false;
    let mut lts: Option<String> = None;
    let mut from_nvmrc = false;
    let mut save = false;

    for arg in args {
        match arg.as_str() {
            "--silent" => silent = true,
            "--delete-prefix" => delete_prefix = true,
            "--" => {}
            "--lts" => lts = Some("*".to_string()),
            "--save" | "-w" => {
                if save {
                    nvm.io.err(messages::SAVE_ONCE);
                    return 6;
                }
                save = true;
            }
            other if other.starts_with("--lts=") => lts = Some(other["--lts=".len()..].to_string()),
            other if other.starts_with("--") => {}
            other => {
                if !other.is_empty() {
                    provided = other.to_string();
                }
            }
        }
    }

    let silent = nvm.silent(silent);
    let version: String;
    if let Some(lts) = &lts {
        let lts = if lts.is_empty() { "*" } else { lts };
        version = local::match_version(nvm, &format!("lts/{lts}")).0;
    } else if provided.is_empty() {
        let (from_rc, _) = nvmrc::rc_version(nvm, silent);
        let mut resolved = String::new();
        if let Some(rc) = from_rc.filter(|rc| !rc.is_empty()) {
            from_nvmrc = true;
            resolved = local::version(nvm, &rc).0;
            provided = rc;
        }
        if resolved.is_empty() {
            nvm.io.err(messages::SEE_HELP_NVMRC);
            return 127;
        }
        version = resolved;
    } else {
        version = local::match_version(nvm, &provided).0;
    }

    if version.is_empty() {
        for line in messages::USE_USAGE {
            nvm.io.err(line);
        }
        return 127;
    }

    if save {
        nvmrc::write_nvmrc(nvm, &version, silent);
    }

    if version == "system" {
        if local::has_system_node(nvm) {
            nvm.quiet(|nvm| deactivate::deactivate(nvm, silent));
            if !silent {
                let node = local::system_node_version(nvm);
                let npm = npm::print_npm_version(nvm);
                nvm.io.echo(&messages::now_using_system_node(&node, &npm));
            }
            return 0;
        } else if local::has_system_iojs(nvm) {
            nvm.quiet(|nvm| deactivate::deactivate(nvm, silent));
            if !silent {
                let iojs = local::which_node(nvm, "iojs")
                    .and_then(|path| local::binary_version(nvm, &path))
                    .unwrap_or_default();
                let npm = npm::print_npm_version(nvm);
                nvm.io.echo(&messages::now_using_system_iojs(&iojs, &npm));
            }
            return 0;
        } else if !silent {
            nvm.io.err(messages::SYSTEM_NOT_FOUND);
        }
        return 127;
    } else if version == "∞" {
        if !silent {
            nvm.io.err(&messages::infinite_loop(&provided));
        }
        return 8;
    }
    if version == "N/A" {
        if !silent {
            local::ensure_version_installed(nvm, &provided, from_nvmrc);
        }
        return 3;
    }
    // nvm returns `$?` after `if ! nvm_ensure_version_installed`, which is 0.
    if local::ensure_version_installed(nvm, &version, from_nvmrc) != 0 {
        return 0;
    }

    let version_dir = match paths::version_path(nvm, &version) {
        Ok(dir) => dir,
        Err(message) => {
            nvm.io.err(&message);
            return 3;
        }
    };
    activate(nvm, &version_dir);
    let backend = nvm.backend;
    backend.switched(&mut nvm.vars, Some(&version));
    // Asks the newly active node for its version while a switch is being
    // paid for anyway, so `nvm current` and the next shell start nothing.
    local::ls_current(nvm);

    if nvm.vars.get("NVM_SYMLINK_CURRENT") == "true"
        && backend.offers(crate::complete::Needs::SymlinkCurrent)
    {
        let current = format!("{}/current", nvm.vars.get("NVM_DIR"));
        let _ = std::fs::remove_file(&current);
        let _ = std::os::unix::fs::symlink(&version_dir, &current);
    }

    let mut output = String::new();
    if !silent {
        let npm = npm::print_npm_version(nvm);
        output = if version::is_iojs(&version) {
            messages::now_using_iojs(version::strip_iojs_prefix(&version), &npm)
        } else {
            messages::now_using_node(&version, &npm)
        };
    }

    let mut command = "nvm use --delete-prefix".to_string();
    if !provided.is_empty() {
        command.push(' ');
        command.push_str(&version);
    }
    if silent {
        command.push_str(" --silent");
    }
    if npm::die_on_prefix(nvm, delete_prefix, &command, &version_dir) != 0 {
        return 11;
    }
    if !output.is_empty() && !silent {
        nvm.io.echo(&output);
    }
    0
}

/// Puts a version's directories on `PATH` and `MANPATH` and sets `NVM_BIN`
/// and `NVM_INC`.
fn activate(nvm: &mut Nvm, version_dir: &str) {
    let path = nvm
        .backend
        .change_path(&nvm.vars, nvm.vars.get("PATH"), "/bin", version_dir);
    nvm.vars.assign("PATH", path);
    if paths::on_path(nvm.vars.get("PATH"), "manpath") {
        let mut manpath = nvm.backend.change_path(
            &nvm.vars,
            nvm.vars.get("MANPATH"),
            "/share/man",
            version_dir,
        );
        // `man` consults its default path only where the list has an empty entry.
        if !(manpath.starts_with(':') || manpath.contains("::") || manpath.ends_with(':')) {
            manpath.push(':');
        }
        nvm.vars.export("MANPATH", manpath);
    }
    nvm.vars.export_existing("PATH");
    nvm.vars.push(Effect::HashReset);
    nvm.vars.export("NVM_BIN", format!("{version_dir}/bin"));
    nvm.vars
        .export("NVM_INC", format!("{version_dir}/include/node"));
}
