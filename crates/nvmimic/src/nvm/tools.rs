//! Running the outside programs nvm runs, found on the calling shell's
//! `PATH` and given its exported variables, as `command <name>` does there.

use super::paths;
use crate::context::Nvm;
use std::os::fd::AsFd;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

/// What a program left behind.
pub struct Ran {
    pub stdout: Vec<u8>,
    pub status: i32,
}

impl Ran {
    /// stdout the way `$( ... )` captures it: without trailing newlines.
    pub fn text(&self) -> String {
        let mut text = String::from_utf8_lossy(&self.stdout).into_owned();
        while text.ends_with('\n') {
            text.pop();
        }
        text
    }

    pub fn ok(&self) -> bool {
        self.status == 0
    }
}

/// A command for `program` as the calling shell would start it, or `None`
/// when it is not on `PATH`.
pub fn command(nvm: &Nvm, program: &str) -> Option<Command> {
    let resolved = if program.contains('/') {
        program.to_string()
    } else {
        paths::which(nvm.vars.get("PATH"), program)?
    };
    let mut command = Command::new(resolved);
    command
        .arg0(program)
        .env_clear()
        .envs(nvm.vars.exported())
        .stdin(Stdio::null());
    if !nvm.vars.get("PWD").is_empty() {
        command.current_dir(nvm.vars.get("PWD"));
    }
    Some(command)
}

/// Runs a program and captures its stdout. What it writes to stderr goes to
/// nvm's stderr. `None` when the program is not on `PATH`.
pub fn run(nvm: &mut Nvm, program: &str, args: &[&str]) -> Option<Ran> {
    let output = command(nvm, program)?.args(args).output().ok()?;
    nvm.io.err_raw(&String::from_utf8_lossy(&output.stderr));
    Some(Ran {
        stdout: output.stdout,
        status: output.status.code().unwrap_or(1),
    })
}

/// [`run`] with stderr dropped, like `2>/dev/null`.
pub fn run_quiet(nvm: &mut Nvm, program: &str, args: &[&str]) -> Option<Ran> {
    nvm.quiet_err(|nvm| run(nvm, program, args))
}

/// Runs a prepared command the way the shell runs a program in the
/// foreground: its stdout and stderr go where nvm's own are going right now.
/// Returns its exit status, or 127 when it cannot be started.
pub fn run_through(nvm: &mut Nvm, mut command: Command) -> i32 {
    use crate::context::ChildStream;
    nvm.io.flush();
    let out = nvm.io.child_stdout();
    let err = nvm.io.child_stderr();
    let stdio = |stream: ChildStream| match stream {
        ChildStream::Inherit => Stdio::inherit(),
        ChildStream::InheritErr => std::io::stderr()
            .as_fd()
            .try_clone_to_owned()
            .map_or_else(|_| Stdio::null(), Stdio::from),
        ChildStream::Forward => Stdio::piped(),
        ChildStream::Discard => Stdio::null(),
    };
    command.stdout(stdio(out)).stderr(stdio(err));
    match command.output() {
        Ok(output) => {
            // Only a forwarded stream was collected. The others are empty.
            nvm.io.out_raw(&String::from_utf8_lossy(&output.stdout));
            nvm.io.err_raw(&String::from_utf8_lossy(&output.stderr));
            output.status.code().unwrap_or(1)
        }
        Err(_) => 127,
    }
}
