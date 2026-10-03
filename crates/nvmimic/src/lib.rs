//! nvmimic: nvm's behavior as a library.
//!
//! The `nvm` command line in `nvmimic-cli` is a thin layer over this crate.
//! [`run`] takes the shell's variables and arguments, prints what nvm would
//! print, and returns nvm's exit status with the changes to make in the shell.

pub mod backend;
pub mod cmd;
pub mod context;
pub mod functions;
pub mod install;
pub mod nvm;
pub mod pin;
pub mod shell;
pub mod text;

use backend::Backend;
use context::{Io, Nvm};
use shell::{Dialect, Effect, Vars};
use std::io::Write;

/// The shell nvm is called from.
pub struct Caller {
    pub vars: Vars,
    pub dialect: Dialect,
    pub stdout_is_tty: bool,
}

/// What one call left behind.
pub struct Outcome {
    pub status: i32,
    pub effects: Vec<Effect>,
}

/// Runs one of nvm's shell functions, `nvm` itself included.
pub fn run(
    caller: Caller,
    backend: &dyn Backend,
    function: &str,
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
    fd3: &mut dyn FnMut(&str),
) -> Outcome {
    let mut nvm = Nvm {
        vars: caller.vars,
        dialect: caller.dialect,
        stdout_is_tty: caller.stdout_is_tty,
        io: Io::new(out, err),
        backend,
    };
    let status = functions::call(&mut nvm, function, args, fd3);
    nvm.io.flush();
    Outcome {
        status,
        effects: nvm.vars.effects().to_vec(),
    }
}
