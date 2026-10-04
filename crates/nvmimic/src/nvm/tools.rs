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

/// `command mkdir -p <dir>`. When the directory cannot be made, the real
/// `mkdir` runs as well, because what it prints is part of nvm's output.
pub fn mkdir_p(nvm: &mut Nvm, dir: &str) -> bool {
    if std::fs::create_dir_all(dir).is_ok() {
        return true;
    }
    run(nvm, "mkdir", &["-p", dir]).is_some_and(|ran| ran.ok())
}

/// Removes one path the way `rm -rf` does: a directory with everything in
/// it, a symlink without what it points to, nothing when there is nothing.
fn remove(path: &str) -> std::io::Result<()> {
    if path.is_empty() {
        return Ok(());
    }
    let result = match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => std::fs::remove_dir_all(path),
        Ok(_) => std::fs::remove_file(path),
        Err(error) => Err(error),
    };
    match result {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// `command rm -rf <paths>`. When something cannot be removed, the real
/// `rm` runs as well, for the same reason as in [`mkdir_p`].
pub fn rm_rf(nvm: &mut Nvm, paths: &[&str]) -> bool {
    let failed: Vec<&str> = paths
        .iter()
        .copied()
        .filter(|path| remove(path).is_err())
        .collect();
    if failed.is_empty() {
        return true;
    }
    let mut args = vec!["-rf"];
    args.extend(failed);
    run(nvm, "rm", &args).is_some_and(|ran| ran.ok())
}

/// [`rm_rf`] with `2>/dev/null`: nothing is printed, so no program runs.
pub fn rm_rf_quiet(paths: &[&str]) -> bool {
    paths.iter().all(|path| remove(path).is_ok())
}

/// Runs a prepared command with `input` on its stdin, like the right-hand
/// side of a pipe. Its stdout and stderr go where nvm's own are going.
pub fn run_fed(nvm: &mut Nvm, mut command: Command, input: &[u8]) -> i32 {
    use crate::context::ChildStream;
    use std::io::Write;
    nvm.io.flush();
    let stdio = |stream: ChildStream| match stream {
        ChildStream::Inherit => Stdio::inherit(),
        ChildStream::InheritErr => std::io::stderr()
            .as_fd()
            .try_clone_to_owned()
            .map_or_else(|_| Stdio::null(), Stdio::from),
        ChildStream::Forward => Stdio::piped(),
        ChildStream::Discard => Stdio::null(),
    };
    command
        .stdin(Stdio::piped())
        .stdout(stdio(nvm.io.child_stdout()))
        .stderr(stdio(nvm.io.child_stderr()));
    let Ok(mut child) = command.spawn() else {
        return 127;
    };
    if let Some(mut stdin) = child.stdin.take() {
        // A program that stops reading early closes the pipe. That is its
        // business, as it is in a shell pipeline.
        let _ = stdin.write_all(input);
    }
    match child.wait_with_output() {
        Ok(output) => {
            nvm.io.out_raw(&String::from_utf8_lossy(&output.stdout));
            nvm.io.err_raw(&String::from_utf8_lossy(&output.stderr));
            output.status.code().unwrap_or(1)
        }
        Err(_) => 127,
    }
}

/// Runs a program as the middle of a pipeline: `input` goes to its stdin
/// and what it prints to stdout comes back. Its stderr goes to nvm's
/// stderr. `None` when the program is not on `PATH`.
pub fn filter(nvm: &mut Nvm, mut command: Command, input: &[u8]) -> Option<Ran> {
    use std::io::Write;
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().ok()?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input);
    }
    let output = child.wait_with_output().ok()?;
    nvm.io.err_raw(&String::from_utf8_lossy(&output.stderr));
    Some(Ran {
        stdout: output.stdout,
        status: output.status.code().unwrap_or(1),
    })
}

/// Runs a program with its stderr joined to its stdout, like `cmd 2>&1`,
/// and returns everything it printed in the order it printed it. `None`
/// when the program is not on `PATH`.
pub fn run_merged(nvm: &Nvm, program: &str, args: &[&str]) -> Option<Ran> {
    use std::io::Read;
    let (mut reader, writer) = std::io::pipe().ok()?;
    let mut command = command(nvm, program)?;
    command
        .args(args)
        .stdout(writer.try_clone().ok()?)
        .stderr(writer);
    let mut child = command.spawn().ok()?;
    // The command still holds this process's ends of the pipe. Reading
    // would never finish while they are open.
    drop(command);
    let mut stdout = Vec::new();
    reader.read_to_end(&mut stdout).ok()?;
    let status = child.wait().ok()?.code().unwrap_or(1);
    Some(Ran { stdout, status })
}
