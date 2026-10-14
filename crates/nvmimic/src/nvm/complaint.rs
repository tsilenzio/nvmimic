//! What the calling shell itself prints when nvm's code asks it for something
//! it cannot do: start a command that is not there, or enter a directory
//! that is not one. Official nvm is shell code, so these lines come from the
//! shell and name a place in `nvm.sh`. nvmimic prints the same lines, with
//! the line numbers taken from the pinned `nvm.sh` when nvmimic is built.
//!
//! Each shell words them its own way. For a missing `npm` on line 3740 of
//! `nvm.sh`, which is line 267 of the function `nvm`:
//!
//! - bash from a script: `/home/me/.nvm/nvm.sh: line 3740: npm: command not found`
//! - bash at a prompt: `bash: npm: command not found`
//! - zsh: `nvm:267: command not found: npm`
//! - dash: `myscript.sh: 3740: npm: not found`

use super::paths;
use crate::context::Nvm;
use crate::shell::Dialect;
use std::path::Path;

/// One line of the pinned `nvm.sh`.
pub struct Site {
    /// Counted from the top of the file, as bash and dash report it.
    pub line: u32,
    /// The function the line is in, which zsh reports.
    pub function: &'static str,
    pub function_line: u32,
}

/// The lines whose errors are reproduced, found by `build.rs`.
pub mod sites {
    use super::Site;
    include!(concat!(env!("OUT_DIR"), "/sites.rs"));
}

/// Why a command word cannot be started.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CannotStart {
    /// A bare name that is nowhere on `PATH`.
    NotFound,
    /// A path to nothing.
    NoSuchFile,
    IsDirectory,
    NotExecutable,
}

/// Checks a command word the way the shell is about to, without starting
/// it. `None` means it can be started.
pub fn cannot_start(nvm: &Nvm, command: &str) -> Option<CannotStart> {
    cannot_start_in(nvm, "", command)
}

/// [`cannot_start`] for a shell that has changed into `dir`, which is where
/// a command word such as `./configure` is looked for.
pub fn cannot_start_in(nvm: &Nvm, dir: &str, command: &str) -> Option<CannotStart> {
    if !command.contains('/') {
        return paths::which(nvm.vars.get("PATH"), command)
            .is_none()
            .then_some(CannotStart::NotFound);
    }
    let path = &Path::new(dir).join(command);
    match std::fs::metadata(path) {
        Err(_) => Some(CannotStart::NoSuchFile),
        Ok(meta) if meta.is_dir() => Some(CannotStart::IsDirectory),
        Ok(_) if !crate::backend::is_executable(path) => Some(CannotStart::NotExecutable),
        Ok(_) => None,
    }
}

/// What bash puts in front of its own messages.
fn bash_prefix(nvm: &Nvm, site: &Site) -> String {
    let shell = &nvm.caller_shell;
    if shell.interactive || shell.script.is_empty() {
        let name = shell.arg0.rsplit('/').next().unwrap_or("");
        format!("{name}: ")
    } else {
        format!("{}: line {}: ", shell.script, site.line)
    }
}

/// zsh shows a line break inside a name as `\n`, and a tab as `\t`.
fn zsh_nice(text: &str) -> String {
    text.replace('\n', "\\n").replace('\t', "\\t")
}

/// The shell's line for a command it could not start.
pub fn not_started(nvm: &Nvm, site: &Site, command: &str, why: CannotStart) -> String {
    use CannotStart::*;
    match nvm.dialect {
        Dialect::Zsh => {
            let reason = match why {
                NotFound => "command not found",
                NoSuchFile => "no such file or directory",
                IsDirectory | NotExecutable => "permission denied",
            };
            let line = site.line - site.function_line;
            format!("{}:{line}: {reason}: {}", site.function, zsh_nice(command))
        }
        Dialect::Bash | Dialect::BashPosix => {
            let reason = match why {
                NotFound => "command not found",
                NoSuchFile => "No such file or directory",
                IsDirectory => "is a directory",
                NotExecutable => "Permission denied",
            };
            format!("{}{command}: {reason}", bash_prefix(nvm, site))
        }
        Dialect::Posix => {
            let reason = match why {
                NotFound | NoSuchFile => "not found",
                IsDirectory | NotExecutable => "Permission denied",
            };
            format!(
                "{}: {}: {command}: {reason}",
                nvm.caller_shell.arg0, site.line
            )
        }
    }
}

/// Why `cd` into a directory would fail, as the system words it, or `None`
/// when it would work. The process itself stays where it is.
pub fn cannot_enter(dir: &str) -> Option<String> {
    let reason = |code: i32| {
        let text = std::io::Error::from_raw_os_error(code).to_string();
        match text.split_once(" (os error") {
            Some((words, _)) => words.to_string(),
            None => text,
        }
    };
    let c_path = std::ffi::CString::new(dir).ok()?;
    match std::fs::metadata(dir) {
        Err(error) => Some(reason(error.raw_os_error().unwrap_or(libc::ENOENT))),
        Ok(meta) if !meta.is_dir() => Some(reason(libc::ENOTDIR)),
        // SAFETY: `c_path` is a valid C string for the length of the call.
        Ok(_) if unsafe { libc::access(c_path.as_ptr(), libc::X_OK) } != 0 => {
            Some(reason(libc::EACCES))
        }
        Ok(_) => None,
    }
}

/// The shell's line for a `cd` inside `nvm_cd` that failed, and the status
/// `cd` returned.
pub fn cd_failed(nvm: &Nvm, dir: &str, reason: &str) -> (String, i32) {
    let site = &sites::CD;
    match nvm.dialect {
        Dialect::Zsh => {
            let line = site.line - site.function_line;
            let mut reason = reason.to_string();
            if let Some(first) = reason.get(..1) {
                reason.replace_range(..1, &first.to_lowercase());
            }
            (
                format!("{}:cd:{line}: {reason}: {}", site.function, zsh_nice(dir)),
                1,
            )
        }
        Dialect::Bash | Dialect::BashPosix => {
            (format!("{}cd: {dir}: {reason}", bash_prefix(nvm, site)), 1)
        }
        Dialect::Posix => (
            format!(
                "{}: {}: cd: can't cd to {dir}",
                nvm.caller_shell.arg0, site.line
            ),
            2,
        ),
    }
}

/// The shell's line for a redirect to a file descriptor that is not open,
/// and the status the redirected command gets.
pub fn bad_descriptor(nvm: &Nvm, site: &Site, descriptor: i32) -> (String, i32) {
    match nvm.dialect {
        Dialect::Zsh => {
            let line = site.line - site.function_line;
            (
                format!(
                    "{}:{line}: {descriptor}: bad file descriptor",
                    site.function
                ),
                1,
            )
        }
        Dialect::Bash | Dialect::BashPosix => (
            format!(
                "{}{descriptor}: Bad file descriptor",
                bash_prefix(nvm, site)
            ),
            1,
        ),
        Dialect::Posix => (
            format!(
                "{}: {}: {descriptor}: Bad file descriptor",
                nvm.caller_shell.arg0, site.line
            ),
            2,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sites_point_into_their_functions() {
        assert_eq!(sites::CD.function, "nvm_cd");
        assert_eq!(sites::CD.line - sites::CD.function_line, 1);
        for site in [
            &sites::DEBUG_SHELL_VERSION,
            &sites::DEBUG_COMMAND,
            &sites::LINK_ABSOLUTE,
            &sites::LINK_RELATIVE,
        ] {
            assert_eq!(site.function, "nvm");
            assert!(site.line > site.function_line);
        }
    }

    #[test]
    fn a_missing_directory_is_named_as_the_system_names_it() {
        assert_eq!(
            cannot_enter("/nonexistent/nvmimic-test").as_deref(),
            Some("No such file or directory")
        );
        assert_eq!(cannot_enter("/"), None);
    }
}
