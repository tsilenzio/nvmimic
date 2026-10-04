//! The `nvmimic` binary. People run it to manage their nvmimic installation.
//! `$NVM_DIR/nvm.sh` runs it through one internal command to do nvm's work.

use nvmimic::Caller;
use nvmimic::backend::NvmDirBackend;
use nvmimic::shell::{self, Dialect, Vars};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

/// Variables `nvm.sh` hands over because nvm reads them whether or not they
/// are exported. Keep in step with `shell/nvm.sh`.
const FORWARDED: &[&str] = &[
    "PATH",
    "MANPATH",
    "NODE_PATH",
    "HOME",
    "PWD",
    "TERM",
    "PREFIX",
    "DEFAULT",
    "NVM_DIR",
    "NVM_BIN",
    "NVM_INC",
    "NVM_CD_FLAGS",
    "NVM_COLORS",
    "NVM_NO_COLORS",
    "NVM_HAS_COLORS",
    "NVM_SILENT",
    "NVM_LTS",
    "NVM_CURRENT",
    "NVM_VERSION_ONLY",
    "NVM_RC_VERSION",
    "NVM_NODEJS_ORG_MIRROR",
    "NVM_IOJS_ORG_MIRROR",
    "NVM_AUTH_HEADER",
    "NVM_SYMLINK_CURRENT",
    "NVM_NO_PROGRESS",
    "NVM_NO_SOURCE_FALLBACK",
    "NVM_OFFLINE",
    "NVM_MAKE_JOBS",
    "NVM_DEBUG",
    "NVM_INSTALL_LOCK",
    "NVM_INSTALL_LOCK_TIMEOUT",
    "NVM_INSTALL_LOCK_STALE",
    "NVM_INSTALL_THIRD_PARTY_HOOK",
    "NODE_VERSION",
    "PROGRESS_BAR",
    "ALIAS",
    "provided_version",
    "SHELL",
    "CC",
    "CXX",
    "TMPDIR",
    "LTS",
    "SKIP_DEFAULT_PACKAGES",
    "PROVIDED_REINSTALL_PACKAGES_FROM",
    "REINSTALL_PACKAGES_FROM",
    "MAKE_CXX",
    "MAKE_SHELL_OVERRIDE",
    "OS_VERSION",
    "VERSION",
];

/// How `shell/nvm.sh` asks for one of nvm's functions. Internal: it is not
/// in the help, and its name and what it expects can change in any release.
/// Keep in step with `xtask/src/gen_shell.rs`.
const HANDOFF: &str = "__internal-shell-handoff";

fn main() -> ExitCode {
    let args: Vec<String> = env::args_os().skip(1).map(lossy).collect();
    match args.split_first() {
        Some((first, rest)) if first == HANDOFF => match rest.split_first() {
            Some((function, rest)) => handoff(function, rest),
            None => ExitCode::from(2),
        },
        _ => nvmimic_command(&args),
    }
}

/// Runs one of nvm's functions for the shell that called `nvm.sh`.
fn handoff(function: &str, rest: &[String]) -> ExitCode {
    // nvm's code runs inside the shell, and bash and dash end a shell that
    // writes to a pipe nobody reads, as in `nvm install 20 | head -1`. Rust
    // programs ignore that signal unless told otherwise. zsh carries on, so
    // nvmimic does too there.
    if dialect() != Dialect::Zsh {
        // SAFETY: sets a signal back to its default before any thread starts.
        unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL) };
    }

    let caller = Caller {
        vars: caller_vars(),
        dialect: dialect(),
        stdout_is_tty: is_tty(1),
        aliases: env::var("__NVMIMIC_ALIASES")
            .unwrap_or_default()
            .split_whitespace()
            .map(str::to_string)
            .collect(),
        command_info: env::var("__NVMIMIC_COMMAND_INFO").unwrap_or_default(),
        shell: nvmimic::context::CallerShell {
            arg0: env::var("__NVMIMIC_ARG0").unwrap_or_default(),
            script: env::var("__NVMIMIC_SOURCE").unwrap_or_default(),
            interactive: env::var("__NVMIMIC_OPTS").is_ok_and(|options| options.contains('i')),
        },
        direct_output: true,
        cache_dir: env::var_os("__NVMIMIC_CACHE")
            .filter(|dir| !dir.is_empty())
            .map(Into::into),
    };
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let mut fd3 = |text: &str| write_fd(3, text);
    let outcome = nvmimic::run(
        caller,
        &NvmDirBackend,
        function,
        rest,
        &mut stdout,
        &mut stderr,
        &mut fd3,
    );

    if let Some(fd) = env::var("__NVMIMIC_FD")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
    {
        let _ = write_fd(fd, &shell::render(&outcome.effects));
    }
    ExitCode::from(outcome.status.rem_euclid(256) as u8)
}

const USAGE: &str = "\
usage: nvmimic <command>

commands:
  self install [--yes] [--dry-run]     install nvmimic into $NVM_DIR, taking over nvm
  self uninstall [--yes] [--dry-run]   remove nvmimic and put nvm's scripts back
  --version                            print nvmimic's own version

--yes answers yes to every question. --dry-run shows what would happen and
changes nothing. self install copies the nvmimic program to
~/.local/bin/nvmimic, or to the path in NVMIMIC_INSTALL_PATH.";

/// The commands people type.
fn nvmimic_command(args: &[String]) -> ExitCode {
    let flags: Vec<&str> = args.iter().skip(2).map(String::as_str).collect();
    let options = nvmimic::install::Options {
        yes: flags.contains(&"--yes"),
        dry_run: flags.contains(&"--dry-run"),
    };
    let unknown = flags
        .iter()
        .find(|flag| !matches!(**flag, "--yes" | "--dry-run"));
    let env = nvmimic::install::Env::from_process();
    let mut ui = nvmimic::install::TerminalUi;
    let result = match (
        args.first().map(String::as_str),
        args.get(1).map(String::as_str),
    ) {
        (Some("--version"), None) => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        (Some("self"), Some(action @ ("install" | "uninstall"))) if unknown.is_none() => {
            if !options.yes && !options.dry_run && !is_tty(0) {
                eprintln!(
                    "nvmimic: this asks questions, and there is no terminal to answer them. Run it again with --yes."
                );
                return ExitCode::from(2);
            }
            if action == "install" {
                match env::current_exe() {
                    Ok(binary) => nvmimic::install::self_install(&env, &binary, &options, &mut ui),
                    Err(error) => Err(format!("cannot find this program's own file: {error}")),
                }
            } else {
                nvmimic::install::self_uninstall(&env, &options, &mut ui)
            }
        }
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("nvmimic: {message}");
            ExitCode::FAILURE
        }
    }
}

fn lossy(value: OsString) -> String {
    value.to_string_lossy().into_owned()
}

/// The shell's variables: everything exported, plus the forwarded ones the
/// shell set without exporting.
fn caller_vars() -> Vars {
    let mut values: BTreeMap<String, String> = BTreeMap::new();
    let mut exported: BTreeSet<String> = BTreeSet::new();
    for (name, value) in env::vars_os() {
        let name = lossy(name);
        if name.starts_with("__NVMIMIC_") {
            continue;
        }
        exported.insert(name.clone());
        values.insert(name, lossy(value));
    }

    if let Ok(set) = env::var("__NVMIMIC_SET") {
        let set: BTreeSet<&str> = set.split_whitespace().collect();
        for name in FORWARDED {
            if set.contains(name) {
                let value = env::var_os(format!("__NVMIMIC_V_{name}"))
                    .map(lossy)
                    .unwrap_or_default();
                values.insert(name.to_string(), value);
            } else {
                values.remove(*name);
                exported.remove(*name);
            }
        }
    }
    Vars::new(values, exported)
}

fn dialect() -> Dialect {
    match env::var("__NVMIMIC_SHELL").as_deref() {
        Ok("zsh") => Dialect::Zsh,
        Ok("bash") => Dialect::Bash,
        Ok("bashposix") => Dialect::BashPosix,
        _ => Dialect::Posix,
    }
}

fn is_tty(fd: i32) -> bool {
    // SAFETY: isatty only inspects the descriptor.
    unsafe { libc::isatty(fd) == 1 }
}

/// Writes to a descriptor the shell opened for us, if it is open.
/// Writes to a file descriptor the shell handed over. `false` when it is
/// not open.
fn write_fd(fd: i32, text: &str) -> bool {
    let mut bytes = text.as_bytes();
    while !bytes.is_empty() {
        // SAFETY: the pointer and length describe `bytes` for the whole call.
        let written = unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) };
        if written <= 0 {
            return std::io::Error::last_os_error().raw_os_error() != Some(libc::EBADF);
        }
        bytes = &bytes[written as usize..];
    }
    true
}
