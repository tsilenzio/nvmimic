//! `nvm which`: the path to an installed version's `node`.

use crate::context::Nvm;
use crate::nvm::{local, nvmrc, paths};
use crate::text::messages;

/// nvm: nvm:which
pub fn run(nvm: &mut Nvm, args: &[String]) -> i32 {
    let mut silent = false;
    let mut provided = String::new();
    for arg in args {
        match arg.as_str() {
            "--silent" => silent = true,
            "--" => {}
            other => provided = other.to_string(),
        }
    }

    let silent = nvm.silent(silent);
    let mut version = String::new();
    if provided.is_empty() {
        if let (Some(rc), _) = nvmrc::rc_version(nvm, silent) {
            provided = rc;
            if !provided.is_empty() {
                version = local::version(nvm, &provided).0;
            }
        }
    } else if provided != "system" {
        version = local::version(nvm, &provided).0;
    } else {
        version = provided.clone();
    }

    if version.is_empty() {
        for line in messages::WHICH_USAGE {
            nvm.io.err(line);
        }
        return 127;
    }

    if version == "system" {
        if local::has_system_iojs(nvm) || local::has_system_node(nvm) {
            // `nvm use system` in a subshell, then `command which node` there.
            let deactivated = nvm
                .backend
                .strip_path(&nvm.vars, nvm.vars.get("PATH"), "/bin");
            return match paths::which(&deactivated, "node") {
                Some(node) => {
                    nvm.io.echo(&node);
                    0
                }
                None => 1,
            };
        }
        nvm.io.err(messages::SYSTEM_NOT_FOUND);
        return 127;
    } else if version == "∞" {
        nvm.io.err(&messages::infinite_loop(&provided));
        return 8;
    }

    // `which` assigns EXIT_CODE without declaring it local, so it stays set.
    let status = local::ensure_version_installed(nvm, &provided, false);
    nvm.vars.assign("EXIT_CODE", status.to_string());
    if status != 0 {
        return status;
    }
    match paths::version_path(nvm, &version) {
        Ok(dir) => {
            nvm.io.echo(&format!("{dir}/bin/node"));
            0
        }
        Err(message) => {
            nvm.io.err(&message);
            3
        }
    }
}
