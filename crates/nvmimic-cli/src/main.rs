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
            bash: env::var("__NVMIMIC_BASH")
                .ok()
                .and_then(|major| major.parse().ok())
                .unwrap_or(0),
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

nvmimic does nvm's job, only faster. In your shell you keep typing `nvm`.
The `nvmimic` command sets nvmimic up, keeps it up to date and removes it
again.

commands:
  self install     set nvmimic up for your nvm folder ($NVM_DIR, usually ~/.nvm)
  self update      get the newest nvmimic
  self rollback    go back to the nvmimic from before the last update
  self uninstall   remove nvmimic again
  help             show this text
  --version        print nvmimic's own version

Add --help to any `self` command to read what it does, step by step.";

const INSTALL_USAGE: &str = "\
usage: nvmimic self install [--yes] [--dry-run]

Sets nvmimic up, so that `nvm` in your shell is nvmimic.

If you have no nvm yet, this makes the nvm folder and puts nvmimic's nvm.sh,
nvm-exec and bash_completion into it. Then it shows the two lines to add to
your shell's startup file.

If nvm is already there, nvmimic takes over from it. Your Node versions,
aliases and settings stay as they are. Only nvm's own three scripts are
swapped. They are kept as a backup, and `nvmimic self uninstall` puts them
back.

Either way, the nvmimic program is copied to ~/.local/bin/nvmimic, so you
can run it again later. Run this again any time to repair the setup, for
example after nvm's own install script was run over it.

  --yes       answer yes to every question
  --dry-run   show what would happen and change nothing

Settings, as environment variables:
  NVM_DIR                the nvm folder, if it isn't ~/.nvm
  NVMIMIC_INSTALL_PATH   where the nvmimic program goes
  NVMIMIC_STATE_DIR, NVMIMIC_CACHE_DIR
                         where nvmimic keeps its other files";

const UNINSTALL_USAGE: &str = "\
usage: nvmimic self uninstall [--yes] [--dry-run]

Removes nvmimic. Your Node versions, aliases and settings stay as they are.

If nvmimic took over from an nvm, that nvm's scripts come back and it works
as it did before. If there was no nvm before nvmimic, the nvm folder keeps
your Node versions but has no `nvm` command any more.

  --yes       answer yes to every question
  --dry-run   show what would happen and change nothing";

const UPDATE_USAGE: &str = "\
usage: nvmimic self update [--yes] [--dry-run]

Gets the newest nvmimic and swaps it in. This updates nvmimic only. It has
nothing to do with which nvm version nvmimic copies, or with your Node
versions.

The download is checked before anything changes: its checksum, that
nvmimic's developer signed it, and that it runs. The nvmimic you have now is
kept, and `nvmimic self rollback` brings it back.

If Homebrew or cargo installed nvmimic, this tells you the command to use
instead.

  --yes       answer yes to every question
  --dry-run   show what would happen and change nothing";

const ROLLBACK_USAGE: &str = "\
usage: nvmimic self rollback [--yes] [--dry-run]

Goes back to the nvmimic you had before the last `nvmimic self update`. The
one you leave is kept in its place, so running this twice gets you back
where you started.

  --yes       answer yes to every question
  --dry-run   show what would happen and change nothing";

/// The help for `nvmimic self <action>`, or for `self` alone.
fn self_usage(action: Option<&str>) -> String {
    match action {
        Some("install") => INSTALL_USAGE.to_string(),
        Some("uninstall") => UNINSTALL_USAGE.to_string(),
        Some("update") => UPDATE_USAGE.to_string(),
        Some("rollback") => ROLLBACK_USAGE.to_string(),
        _ => format!("{INSTALL_USAGE}\n\n{UPDATE_USAGE}\n\n{ROLLBACK_USAGE}\n\n{UNINSTALL_USAGE}"),
    }
}

fn is_help(word: &str) -> bool {
    matches!(word, "help" | "--help" | "-h")
}

/// The commands people type.
fn nvmimic_command(args: &[String]) -> ExitCode {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    let (action, flags) = match words.as_slice() {
        [] => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        [word] if is_help(word) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        ["--version"] => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        ["self"] => {
            println!("{}", self_usage(None));
            return ExitCode::SUCCESS;
        }
        ["self", word] if is_help(word) => {
            println!("{}", self_usage(None));
            return ExitCode::SUCCESS;
        }
        [
            "self",
            action @ ("install" | "uninstall" | "update" | "rollback"),
            flags @ ..,
        ] => (*action, flags),
        _ => {
            eprintln!(
                "nvmimic: there is no command `{}`.\n\n{USAGE}",
                words.join(" ")
            );
            return ExitCode::from(2);
        }
    };
    if flags.iter().any(|flag| is_help(flag)) {
        println!("{}", self_usage(Some(action)));
        return ExitCode::SUCCESS;
    }
    if let Some(unknown) = flags
        .iter()
        .find(|flag| !matches!(**flag, "--yes" | "--dry-run"))
    {
        eprintln!(
            "nvmimic: `self {action}` has no option `{unknown}`.\n\n{}",
            self_usage(Some(action))
        );
        return ExitCode::from(2);
    }
    let options = nvmimic::install::Options {
        yes: flags.contains(&"--yes"),
        dry_run: flags.contains(&"--dry-run"),
    };
    if !options.yes && !options.dry_run && !is_tty(0) {
        eprintln!(
            "nvmimic: this asks questions, and there is no terminal to answer them. Run it again with --yes."
        );
        return ExitCode::from(2);
    }
    let env = nvmimic::install::Env::from_process();
    let mut ui = nvmimic::install::TerminalUi;
    let result = match action {
        "install" => match env::current_exe() {
            Ok(binary) => nvmimic::install::self_install(&env, &binary, &options, &mut ui),
            Err(error) => Err(format!("cannot find this program's own file: {error}")),
        },
        "update" => {
            let source = nvmimic::update::Source::from_process();
            nvmimic::update::self_update(&env, &source, &options, &mut ui)
        }
        "rollback" => nvmimic::update::self_rollback(&env, &options, &mut ui),
        _ => nvmimic::install::self_uninstall(&env, &options, &mut ui),
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
