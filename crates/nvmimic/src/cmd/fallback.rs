//! Commands and helpers nvmimic has not ported yet run in real nvm, which is
//! built into the binary, inside a shell of the caller's kind. Every
//! variable that run changes comes back to the caller, the same way changes
//! from ported commands do.
//!
//! This keeps every nvm command working while porting is under way. None of
//! the everyday commands take this path.

use crate::context::Nvm;
use crate::shell::Dialect;
use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

/// The pinned nvm.sh, copied unchanged from nvm (https://github.com/nvm-sh/nvm),
/// MIT licensed: Copyright (c) 2010 Tim Caswell, Copyright (c) 2014 Jordan Harband.
const NVM_SH: &str = include_str!("../../../../tests/reference/nvm-v0.40.8/nvm.sh");

/// Shell variables that change on their own or belong to the shell rather
/// than to anything nvm did.
const IGNORED: &[&str] = &[
    "_",
    "BASHPID",
    "BASH_ARGC",
    "BASH_ARGV",
    "BASH_COMMAND",
    "BASH_LINENO",
    "BASH_SOURCE",
    "BASH_SUBSHELL",
    "COLUMNS",
    "EPOCHREALTIME",
    "EPOCHSECONDS",
    "FUNCNAME",
    "HISTCMD",
    "LINENO",
    "LINES",
    "OLDPWD",
    "OPTIND",
    "PIPESTATUS",
    "PPID",
    "RANDOM",
    "SECONDS",
    "SRANDOM",
    "TTYIDLE",
    "ZSH_EVAL_CONTEXT",
    "zsh_eval_context",
    "funcstack",
    "funcfiletrace",
    "funcsourcetrace",
    "functrace",
    "status",
    "pipestatus",
    "argv",
    "ARGC",
    "SHLVL",
    // zsh ties these arrays to PATH-style scalars, which carry the change.
    "path",
    "manpath",
    "fpath",
    "cdpath",
    "mailpath",
    "module_path",
    "fignore",
    "psvar",
];

/// Writes every shell variable, `NAME=value` NUL-terminated, to `$1`, and
/// the shell's export list to `$1.exported`.
const SNAPSHOT: &str = r#"
__nvmimic_snapshot() {
  ( set ) 2>/dev/null | while IFS= read -r __nvmimic_line; do
    case "$__nvmimic_line" in
      [A-Za-z_]*=*)
        __nvmimic_name="${__nvmimic_line%%=*}"
        case "$__nvmimic_name" in
          *[!A-Za-z0-9_]* | __nvmimic_* | __NVMIMIC_*) ;;
          *) eval "printf '%s=%s\0' \"\$__nvmimic_name\" \"\${$__nvmimic_name-}\"" ;;
        esac
      ;;
    esac
  done > "$1"
  export -p > "$1.exported"
}
"#;

pub fn run_nvm(nvm: &mut Nvm, args: &[String]) -> i32 {
    run(nvm, "nvm", args)
}

/// Runs one of nvm's helper functions, for helpers not ported yet.
pub fn run_function(nvm: &mut Nvm, name: &str, args: &[String]) -> i32 {
    run(nvm, name, args)
}

/// The shell to run real nvm in, matching the caller's.
fn shell_command(dialect: Dialect) -> Command {
    match dialect {
        Dialect::Zsh => {
            let mut command = Command::new("zsh");
            command.arg("-f");
            command
        }
        Dialect::Bash => Command::new("bash"),
        Dialect::BashPosix => {
            let mut command = Command::new("bash");
            command.arg("--posix");
            command
        }
        Dialect::Posix => {
            if std::path::Path::new("/bin/dash").is_file() {
                Command::new("/bin/dash")
            } else {
                Command::new("/bin/sh")
            }
        }
    }
}

fn run(nvm: &mut Nvm, function: &str, args: &[String]) -> i32 {
    let dir = format!("{}/.nvmimic/fallback", nvm.vars.get("NVM_DIR"));
    let script = format!("{dir}/nvm-{}.sh", crate::pin::VERSION);
    let up_to_date = std::fs::read(&script).is_ok_and(|bytes| bytes == NVM_SH.as_bytes());
    if !up_to_date {
        let _ = std::fs::create_dir_all(&dir);
        if let Err(error) = std::fs::write(&script, NVM_SH) {
            let reason = crate::text::messages::strerror(&error);
            nvm.io
                .err(&format!("nvmimic: cannot write {script}: {reason}"));
            return 1;
        }
    }
    let before = format!("{dir}/before.{}", std::process::id());
    let after = format!("{dir}/after.{}", std::process::id());

    // Variables the caller set without exporting are set the same way here.
    let exported: BTreeSet<String> = nvm.vars.exported_names().into_iter().collect();
    let unexported: String = nvm
        .vars
        .all()
        .into_iter()
        .filter(|(name, _)| !exported.contains(name))
        .map(|(name, value)| format!("{name}={}\n", crate::shell::quote(&value)))
        .collect();
    let program = format!(
        r#"{unexported}{SNAPSHOT}
. "$0" --no-use
shift
__nvmimic_snapshot "$__NVMIMIC_BEFORE"
"$@"
__nvmimic_rc=$?
__nvmimic_snapshot "$__NVMIMIC_AFTER"
exit $__nvmimic_rc
"#
    );

    nvm.io.flush();
    let mut command = shell_command(nvm.dialect);
    command
        .arg("-c")
        .arg(program)
        .arg(&script)
        // dash's `.` passes no arguments, so nvm.sh reads these instead.
        .arg("--no-use")
        .arg(function)
        .args(args)
        .env_clear()
        .envs(nvm.vars.exported())
        .env("__NVMIMIC_BEFORE", &before)
        .env("__NVMIMIC_AFTER", &after);
    if !nvm.vars.get("PWD").is_empty() {
        command.current_dir(nvm.vars.get("PWD"));
    }
    let status = match command.status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(error) => {
            let reason = crate::text::messages::strerror(&error);
            nvm.io
                .err(&format!("nvmimic: cannot start a shell: {reason}"));
            return 1;
        }
    };

    let (old, _) = read_snapshot(&before);
    let (new, exported) = read_snapshot(&after);
    for path in [&before, &after] {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{path}.exported"));
    }
    if new.is_empty() {
        return status;
    }
    for name in old.keys().filter(|name| !new.contains_key(*name)) {
        nvm.vars.unset(name);
    }
    for (name, value) in &new {
        if old.get(name) == Some(value) {
            continue;
        }
        if exported.contains(name) {
            nvm.vars.export(name, value.clone());
        } else {
            nvm.vars.assign(name, value.clone());
        }
    }
    status
}

/// Reads one snapshot: the variables, and the names `export -p` listed.
fn read_snapshot(path: &str) -> (BTreeMap<String, String>, BTreeSet<String>) {
    let bytes = std::fs::read(path).unwrap_or_default();
    let vars = String::from_utf8_lossy(&bytes)
        .split('\0')
        .filter_map(|record| record.split_once('='))
        .filter(|(name, _)| !IGNORED.contains(name))
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect();
    let listing = std::fs::read_to_string(format!("{path}.exported")).unwrap_or_default();
    let exported = listing
        .lines()
        .filter_map(|line| {
            // zsh lists tied variables such as MANPATH as `export -T NAME name=(...)`.
            let rest = line
                .strip_prefix("declare -x ")
                .or_else(|| line.strip_prefix("export -T "))
                .or_else(|| line.strip_prefix("export "))
                .or_else(|| line.strip_prefix("typeset -x "))?;
            let name = rest.split(['=', ' ']).next()?;
            name.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
                .then(|| name.to_string())
        })
        .collect();
    (vars, exported)
}
