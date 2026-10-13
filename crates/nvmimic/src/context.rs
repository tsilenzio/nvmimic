//! Everything one nvm call works with: the shell's variables, where output
//! goes, and the backend that holds installed versions and aliases.

use crate::backend::Backend;
use crate::shell::{Dialect, Vars};
use std::io::Write;

/// Where nvm's output goes. Muting a stream is how `>/dev/null` and
/// `2>/dev/null` on a call inside nvm are expressed.
pub struct Io<'a> {
    out: &'a mut dyn Write,
    err: &'a mut dyn Write,
    out_muted: u32,
    err_muted: u32,
    out_to_err: u32,
    /// Whether `out` is this process's own stdout, so a program nvm starts
    /// can write to it directly.
    direct_out: bool,
    /// The same for `err` and this process's stderr.
    direct_err: bool,
}

/// Where a program nvm starts should send one of its output streams.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildStream {
    /// Straight to this process's own stream, as it happens.
    Inherit,
    /// To this process's stderr, for a stdout that nvm redirected there.
    InheritErr,
    /// Collected, then written through [`Io`].
    Forward,
    Discard,
}

impl<'a> Io<'a> {
    pub fn new(out: &'a mut dyn Write, err: &'a mut dyn Write) -> Io<'a> {
        Io {
            out,
            err,
            out_muted: 0,
            err_muted: 0,
            out_to_err: 0,
            direct_out: false,
            direct_err: false,
        }
    }

    /// Says that `out` and `err` are the process's real stdout and stderr.
    pub fn direct(mut self) -> Io<'a> {
        self.direct_out = true;
        self.direct_err = true;
        self
    }

    /// Where a started program's stdout should go.
    pub fn child_stdout(&self) -> ChildStream {
        if self.out_to_err > 0 {
            match self.child_stderr() {
                ChildStream::Inherit => ChildStream::InheritErr,
                other => other,
            }
        } else if self.out_muted > 0 {
            ChildStream::Discard
        } else if self.direct_out {
            ChildStream::Inherit
        } else {
            ChildStream::Forward
        }
    }

    /// Where a started program's stderr should go.
    pub fn child_stderr(&self) -> ChildStream {
        if self.err_muted > 0 {
            ChildStream::Discard
        } else if self.direct_err {
            ChildStream::Inherit
        } else {
            ChildStream::Forward
        }
    }

    /// Writes bytes to stdout exactly as given.
    pub fn out_raw(&mut self, text: &str) {
        if self.out_to_err > 0 {
            self.err_raw(text);
        } else if self.out_muted == 0 {
            let _ = self.out.write_all(text.as_bytes());
        }
    }

    /// Writes bytes to stderr exactly as given.
    pub fn err_raw(&mut self, text: &str) {
        if self.err_muted == 0 {
            let _ = self.err.write_all(text.as_bytes());
        }
    }

    /// nvm: nvm_echo
    pub fn echo(&mut self, line: &str) {
        self.out_raw(line);
        self.out_raw("\n");
    }

    /// nvm: nvm_err
    pub fn err(&mut self, line: &str) {
        self.err_raw(line);
        self.err_raw("\n");
    }

    pub fn flush(&mut self) {
        let _ = self.out.flush();
        let _ = self.err.flush();
    }
}

/// Things one call of nvmimic looks up once and reuses, where nvm would
/// repeat the work without any visible difference.
#[derive(Default)]
pub struct Cache {
    pub curl_compression: Option<bool>,
    /// What `nvm_get_os` and `nvm_get_arch` found.
    pub os: Option<String>,
    pub arch: Option<String>,
    /// `index.tab` bodies by URL.
    pub index_tab: std::collections::HashMap<String, String>,
}

pub struct Nvm<'a> {
    pub vars: Vars,
    pub cache: std::rc::Rc<std::cell::RefCell<Cache>>,
    pub dialect: Dialect,
    /// `[ -t 1 ]` in the calling shell.
    pub stdout_is_tty: bool,
    pub io: Io<'a>,
    pub backend: &'a dyn Backend,
    /// Names that are aliases in the calling shell, among the ones the shell
    /// side reports. nvm skips a checksum program that is aliased.
    pub aliases: std::rc::Rc<Vec<String>>,
    /// nvm's `nvm_command_info` for the programs `nvm debug` reports on, as
    /// the calling shell answered it. See `cmd::debug`.
    pub command_info: std::rc::Rc<String>,
    pub caller_shell: std::rc::Rc<CallerShell>,
    /// Where answers are remembered between calls, see `nvm::remembered`.
    pub cache_dir: Option<std::rc::Rc<std::path::PathBuf>>,
    /// The program whose shell functions made this call.
    pub identity: &'static crate::identity::Identity,
    /// Whether this is a loading shell switching by itself, and not a
    /// command someone ran. See [`crate::backend::Reason`].
    pub reason: crate::backend::Reason,
}

/// What the calling shell goes by when it reports an error of its own. See
/// `nvm::complaint`.
#[derive(Default)]
pub struct CallerShell {
    /// `$0`: the shell's name or the script it is running.
    pub arg0: String,
    /// The path `nvm.sh` was sourced by, which bash names in its messages.
    /// Empty in other shells.
    pub script: String,
    /// Whether the shell is reading commands from a person.
    pub interactive: bool,
    /// bash's major version, or 0 in another shell or when it isn't known.
    pub bash: u32,
}

/// A variable's value before a `local` declaration hid it.
pub struct Saved(Option<String>);

impl<'a> Nvm<'a> {
    /// `local NAME`: in bash and zsh the variable starts empty, in dash it
    /// keeps the caller's value. Pass the result to [`Nvm::restore`].
    pub fn local(&mut self, name: &str) -> Saved {
        let saved = Saved(self.vars.get_opt(name).map(str::to_string));
        if self.dialect != crate::shell::Dialect::Posix {
            self.vars.unset_quiet(name);
        }
        saved
    }

    /// Runs `body` with `names` declared `local`: whatever it assigns to them
    /// stays inside, as it does when an nvm function returns.
    pub fn scoped<T>(&mut self, names: &[&str], body: impl FnOnce(&mut Nvm<'a>) -> T) -> T {
        let mark = self.vars.mark();
        let saved: Vec<(String, Saved)> = names
            .iter()
            .map(|name| (name.to_string(), self.local(name)))
            .collect();
        let result = body(self);
        for (name, value) in saved {
            self.vars.forget_since(mark, &name);
            self.restore(&name, value);
        }
        result
    }

    /// `NAME=value func`, run in the current shell. bash in POSIX mode, which
    /// is macOS's `/bin/sh`, keeps such assignments after the function
    /// returns, as exported globals, so nvm leaves them behind there.
    pub fn prefix_assignments(&mut self, assignments: &[(&str, &str)]) {
        if self.dialect == crate::shell::Dialect::BashPosix {
            for (name, value) in assignments {
                self.vars.export(name, *value);
            }
        }
    }

    /// Whether `--silent` is in effect for a command that declares
    /// `local NVM_SILENT`. Under dash the local keeps an inherited value, so
    /// an `NVM_SILENT=1` from the caller silences the command there too.
    pub fn silent(&self, flag: bool) -> bool {
        flag || (self.dialect == crate::shell::Dialect::Posix && self.vars.get("NVM_SILENT") == "1")
    }

    /// Runs `body` in a copy of the shell state, as `$( ... )` does: stdout
    /// still goes where it was going, but variable changes are thrown away.
    pub fn subshell<T>(&mut self, body: impl FnOnce(&mut Nvm<'a>) -> T) -> T {
        let saved = self.vars.clone();
        let result = body(self);
        self.vars = saved;
        result
    }

    /// Puts back what [`Nvm::local`] hid, as returning from the function does.
    pub fn restore(&mut self, name: &str, saved: Saved) {
        match saved.0 {
            Some(value) => self.vars.set_quiet(name, &value),
            None => self.vars.unset_quiet(name),
        }
    }

    /// Runs `body` with stdout discarded, like `cmd >/dev/null`.
    pub fn quiet_out<T>(&mut self, body: impl FnOnce(&mut Nvm<'a>) -> T) -> T {
        self.io.out_muted += 1;
        let result = body(self);
        self.io.out_muted -= 1;
        result
    }

    /// Runs `body` with stdout sent to stderr, like `>&2 cmd`.
    pub fn to_stderr<T>(&mut self, body: impl FnOnce(&mut Nvm<'a>) -> T) -> T {
        self.io.out_to_err += 1;
        let result = body(self);
        self.io.out_to_err -= 1;
        result
    }

    /// Runs `body` with stderr discarded, like `cmd 2>/dev/null`.
    pub fn quiet_err<T>(&mut self, body: impl FnOnce(&mut Nvm<'a>) -> T) -> T {
        self.io.err_muted += 1;
        let result = body(self);
        self.io.err_muted -= 1;
        result
    }

    /// Runs `body` with both streams discarded, like `cmd >/dev/null 2>&1`.
    pub fn quiet<T>(&mut self, body: impl FnOnce(&mut Nvm<'a>) -> T) -> T {
        self.quiet_out(|nvm| nvm.quiet_err(body))
    }

    /// Runs `body` as `$( ... )` runs a command: what it prints to stdout is
    /// returned instead of shown, with trailing newlines removed, and its
    /// variable changes are dropped. stderr still reaches the terminal.
    pub fn capture(&mut self, body: impl FnOnce(&mut Nvm<'_>)) -> String {
        let mut buffer: Vec<u8> = Vec::new();
        {
            let err_muted = self.io.err_muted;
            let direct_err = self.io.direct_err;
            let aliases = self.aliases.clone();
            let command_info = self.command_info.clone();
            let caller_shell = self.caller_shell.clone();
            let cache_dir = self.cache_dir.clone();
            let mut err = ErrForward(&mut self.io);
            let mut inner = Nvm {
                vars: self.vars.snapshot(),
                cache: self.cache.clone(),
                dialect: self.dialect,
                stdout_is_tty: false,
                io: Io {
                    out: &mut buffer,
                    err: &mut err,
                    out_muted: 0,
                    err_muted,
                    out_to_err: 0,
                    // stdout is collected here. stderr is not: a program
                    // started in here may still write to the real one, which
                    // is how curl's progress bar shows while it downloads.
                    direct_out: false,
                    direct_err,
                },
                backend: self.backend,
                aliases,
                command_info,
                caller_shell,
                cache_dir,
                identity: self.identity,
                reason: self.reason,
            };
            body(&mut inner);
        }
        let mut text = String::from_utf8_lossy(&buffer).into_owned();
        while text.ends_with('\n') {
            text.pop();
        }
        text
    }
}

/// Lets a captured call write its stderr through the caller's stderr.
struct ErrForward<'b, 'a>(&'b mut Io<'a>);

impl Write for ErrForward<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.err_raw(&String::from_utf8_lossy(buf));
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
