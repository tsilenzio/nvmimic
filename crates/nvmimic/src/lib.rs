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
    /// Names that are aliases in the shell, see [`context::Nvm::aliases`].
    pub aliases: Vec<String>,
    /// What the shell found out about the programs `nvm debug` reports on,
    /// see [`context::Nvm::command_info`].
    pub command_info: String,
    /// Whether `out` and `err` passed to [`run`] are the process's own
    /// stdout and stderr. Programs nvm starts (curl with its progress bar,
    /// npm, make) then write to the terminal directly.
    pub direct_output: bool,
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
        cache: Default::default(),
        dialect: caller.dialect,
        stdout_is_tty: caller.stdout_is_tty,
        io: if caller.direct_output {
            Io::new(out, err).direct()
        } else {
            Io::new(out, err)
        },
        backend,
        aliases: std::rc::Rc::new(caller.aliases),
        command_info: std::rc::Rc::new(caller.command_info),
    };
    let status = functions::call(&mut nvm, function, args, fd3);
    nvm.io.flush();
    Outcome {
        status,
        effects: nvm.vars.effects().to_vec(),
    }
}
