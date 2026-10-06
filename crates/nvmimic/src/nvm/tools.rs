//! Running the outside programs nvm runs, found on the calling shell's
//! `PATH` and given its exported variables, as `command <name>` does there.

use super::paths;
use crate::backend::remove_tree as remove;
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
    let mut command = if needs_shell(&resolved) {
        let mut shell = Command::new("/bin/sh");
        shell.arg(&resolved);
        shell
    } else {
        let mut direct = Command::new(&resolved);
        direct.arg0(program);
        direct
    };
    command
        .env_clear()
        .envs(nvm.vars.exported())
        .stdin(Stdio::null());
    if !nvm.vars.get("PWD").is_empty() {
        command.current_dir(nvm.vars.get("PWD"));
    }
    Some(command)
}

/// Whether a file is a script with no `#!` line. A shell runs such a file
/// itself. macOS does the same for any program that starts one, by handing
/// it to `/bin/sh`. Linux refuses to start it, so nvmimic hands it over.
fn needs_shell(path: &str) -> bool {
    use std::io::Read;
    if cfg!(target_os = "macos") {
        return false;
    }
    let mut start = [0; 4];
    let Ok(read) = std::fs::File::open(path).and_then(|mut file| file.read(&mut start)) else {
        return false;
    };
    let start = &start[..read];
    !start.starts_with(b"#!") && !start.starts_with(b"\x7fELF")
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

/// Writes `input` to a program's stdin and closes it.
///
/// A program that stops reading early, or exits before it reads anything,
/// closes the pipe. In a shell pipeline that ends whatever was writing on
/// the left and the shell carries on with the program's status. nvmimic is
/// the writer here and has to carry on too, so the signal that ends a
/// process writing to a closed pipe is switched off for the write and then
/// put back as it was. GNU grep exits on a bad option before it reads.
fn feed(stdin: Option<std::process::ChildStdin>, input: &[u8]) {
    use std::io::Write;
    let Some(mut stdin) = stdin else {
        return;
    };
    // SAFETY: changes how one signal is handled and restores it below.
    // nvmimic runs no other thread that could write to a pipe meanwhile.
    let before = unsafe { libc::signal(libc::SIGPIPE, libc::SIG_IGN) };
    let _ = stdin.write_all(input);
    drop(stdin);
    // SAFETY: puts back the handling read just above.
    unsafe { libc::signal(libc::SIGPIPE, before) };
}

/// Runs a prepared command with `input` on its stdin, like the right-hand
/// side of a pipe. Its stdout and stderr go where nvm's own are going.
pub fn run_fed(nvm: &mut Nvm, mut command: Command, input: &[u8]) -> i32 {
    use crate::context::ChildStream;
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
    feed(child.stdin.take(), input);
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
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().ok()?;
    feed(child.stdin.take(), input);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A reader that exits without reading must not take the writer with
    /// it. More is written than a pipe holds, so the write always meets
    /// the closed pipe, however the two processes are scheduled. A wrong
    /// `feed` ends this test program instead of failing an assertion.
    #[test]
    fn feeding_a_program_that_already_exited_is_survived() {
        // SAFETY: the test sets the signal as nvmimic's hand-off does under
        // bash and dash, and puts Rust's own setting back at the end.
        let rust_default = unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL) };
        let mut child = Command::new("/bin/sh")
            .args(["-c", "exit 2"])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        feed(child.stdin.take(), &vec![b'x'; 1 << 20]);
        assert_eq!(child.wait().unwrap().code(), Some(2));
        // SAFETY: as above.
        let after = unsafe { libc::signal(libc::SIGPIPE, rust_default) };
        assert_eq!(after, libc::SIG_DFL, "feed left the signal changed");
    }
}
