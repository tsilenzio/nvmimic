//! `nvm deactivate`: takes nvm's directories back out of `PATH`, `MANPATH`
//! and `NODE_PATH`.

use crate::context::Nvm;
use crate::shell::Effect;
use crate::text::messages;

/// nvm: nvm:deactivate
pub fn run(nvm: &mut Nvm, args: &[String]) -> i32 {
    let silent = nvm.silent(args.iter().any(|arg| arg == "--silent"));
    deactivate(nvm, silent);
    0
}

/// `nvm deactivate >/dev/null 2>&1`, as nvm runs it before giving up.
pub fn deactivate_quietly(nvm: &mut Nvm) {
    nvm.quiet(|nvm| deactivate(nvm, false));
}

pub fn deactivate(nvm: &mut Nvm, silent: bool) {
    let nvm_dir = nvm.vars.get("NVM_DIR").to_string();
    let path = nvm.vars.get("PATH").to_string();
    let new_path = nvm.backend.strip_path(&nvm.vars, &path, "/bin");
    if path == new_path {
        if !silent {
            nvm.io.err(&messages::could_not_find_bin(&nvm_dir));
        }
    } else {
        nvm.vars.export("PATH", new_path);
        nvm.vars.push(Effect::HashReset);
        if !silent {
            nvm.io.echo(&messages::removed_bin(&nvm_dir));
        }
    }

    let manpath = nvm.vars.get("MANPATH").to_string();
    if !manpath.is_empty() {
        let new_manpath = nvm.backend.strip_path(&nvm.vars, &manpath, "/share/man");
        if manpath == new_manpath {
            if !silent {
                nvm.io.err(&messages::could_not_find_man(&nvm_dir));
            }
        } else {
            // `man` treats both of these as unset anyway, so nvm leaves nothing.
            match new_manpath.as_str() {
                "" | ":" => nvm.vars.unset("MANPATH"),
                _ => nvm.vars.export("MANPATH", new_manpath),
            }
            if !silent {
                nvm.io.echo(&messages::removed_man(&nvm_dir));
            }
        }
    }

    let node_path = nvm.vars.get("NODE_PATH").to_string();
    if !node_path.is_empty() {
        let new_node_path = nvm
            .backend
            .strip_path(&nvm.vars, &node_path, "/lib/node_modules");
        if node_path != new_node_path {
            nvm.vars.export("NODE_PATH", new_node_path);
            if !silent {
                nvm.io.echo(&messages::removed_node_modules(&nvm_dir));
            }
        }
    }
    nvm.vars.unset("NVM_BIN");
    nvm.vars.unset("NVM_INC");
}
