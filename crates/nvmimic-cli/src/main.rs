//! The `nvmimic` binary. People run it to manage their nvmimic installation.
//! `$NVM_DIR/nvm.sh` runs `nvmimic init` to get the shell functions, and the
//! functions run it through one internal command to do nvm's work.

use nvmimic::Caller;
use nvmimic::backend::NvmDirBackend;
use nvmimic::init::Named;
use nvmimic::shell::{self, Dialect, Vars};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

/// Variables the shell functions hand over because nvm reads them whether
/// or not they are exported. `cargo xtask gen-shell` reads this list.
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

/// How the shell functions ask for one of nvm's functions. Internal: it is not
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
        let _ = write_fd(fd, &(refresh() + &shell::render(&outcome.effects)));
    }
    ExitCode::from(outcome.status.rem_euclid(256) as u8)
}

/// The path the shell functions should run this program by. `nvm.sh` and
/// the functions say which path they used, and that is the one to keep: a
/// path a package manager links to the newest version stays right after an
/// upgrade. Typed as `nvmimic init`, it is wherever the program was found.
fn program() -> Option<std::path::PathBuf> {
    let absolute =
        |path: OsString| Some(std::path::PathBuf::from(path)).filter(|path| path.is_absolute());
    env::var_os("__NVMIMIC_PROGRAM")
        .and_then(absolute)
        .or_else(|| env::args_os().next().and_then(absolute))
        .or_else(|| env::current_exe().ok())
}

/// New shell functions for a shell that holds another version's, which
/// happens when nvmimic was updated while the shell was open. Empty when
/// the shell is up to date. The shell runs it with the rest of what a call
/// sends back. The cache folder stays the one the shell started with.
fn refresh() -> String {
    let held = env::var("__NVMIMIC_CODE").unwrap_or_default();
    if held.is_empty() || held == env!("CARGO_PKG_VERSION") {
        return String::new();
    }
    let Some(binary) = program() else {
        return String::new();
    };
    let cache = env::var_os("__NVMIMIC_CACHE")
        .filter(|dir| !dir.is_empty())
        .map(std::path::PathBuf::from);
    nvmimic::install::functions_text(&nvmimic::install::Bound {
        binary: &binary,
        cache: cache.as_deref(),
    })
}

/// `nvmimic init [shell]`: prints the shell code that loads nvmimic.
fn init(named: Option<Named>) -> ExitCode {
    let Some(binary) = program() else {
        eprintln!("nvmimic: could not work out where the nvmimic program is.");
        return ExitCode::FAILURE;
    };
    // An installation remembers where its cache folder is. Without one the
    // folder comes from the environment, as it would for `self install`.
    let dirs = nvmimic::dirs::Dirs::from_process();
    let manifest = dirs.as_ref().and_then(nvmimic::install::Manifest::read);
    let cache = dirs.as_ref().map(|dirs| {
        manifest
            .as_ref()
            .and_then(|manifest| manifest.cache.clone())
            .unwrap_or_else(|| dirs.cache.clone())
    });
    // `nvm.sh` describes the shell it runs `init` for. Typed directly,
    // nothing does, and there is no `nvm.sh` whose folder could be the nvm
    // folder either. That folder is then the one nvmimic was set up for,
    // or `~/.nvm`.
    let described = init_shell(cache.clone());
    let nvm_dir = match described {
        Some(_) => None,
        None => manifest
            .and_then(|manifest| manifest.nvm_dir)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".nvm"))),
    };
    let load = match (described, named) {
        (Some(shell), _) => nvmimic::init::load(FORWARDED, shell, &NvmDirBackend),
        (None, Some(named)) => {
            let mut seen = exported();
            if let Some(nvm_dir) = &nvm_dir
                && seen.get("NVM_DIR").is_none_or(String::is_empty)
            {
                seen.insert("NVM_DIR".to_string(), nvm_dir.display().to_string());
            }
            nvmimic::init::load_named(FORWARDED, named, &seen, cache.as_deref(), &NvmDirBackend)
        }
        (None, None) => None,
    };
    print!(
        "{}",
        nvmimic::install::init_text(
            &nvmimic::install::Bound {
                binary: &binary,
                cache: cache.as_deref(),
            },
            nvm_dir.as_deref(),
            load.as_deref(),
        )
    );
    ExitCode::SUCCESS
}

/// The exported variables, without the ones `nvm.sh` and the shell
/// functions pass for one start of the program.
fn exported() -> BTreeMap<String, String> {
    env::vars_os()
        .map(|(name, value)| (lossy(name), lossy(value)))
        .filter(|(name, _)| !name.starts_with("__NVMIMIC_"))
        .collect()
}

/// The shell `nvm.sh` says it is running `init` for. `None` when `init` was
/// typed directly.
fn init_shell(cache_dir: Option<std::path::PathBuf>) -> Option<nvmimic::init::Shell> {
    let hint = |name: &str| env::var(name).unwrap_or_default();
    env::var_os("__NVMIMIC_OPTS")?;
    let bash: u32 = hint("__NVMIMIC_BASH").parse().unwrap_or(0);
    let dialect = if !hint("__NVMIMIC_ZSH").is_empty() {
        Dialect::Zsh
    } else if bash == 0 {
        Dialect::Posix
    } else if hint("__NVMIMIC_SHELLOPTS").split(':').any(|o| o == "posix") {
        Dialect::BashPosix
    } else {
        Dialect::Bash
    };
    Some(nvmimic::init::Shell {
        env: exported(),
        dialect,
        caller: nvmimic::context::CallerShell {
            arg0: hint("__NVMIMIC_ARG0"),
            script: hint("__NVMIMIC_SOURCE"),
            interactive: hint("__NVMIMIC_OPTS").contains('i'),
            bash,
        },
        cache_dir,
    })
}

const INIT_USAGE: &str = "\
usage: nvmimic init [bash|zsh|sh|dash]

Prints the shell code that makes `nvm` work in a shell: nvm's functions, and
then the lines that switch to your default Node version. A shell has to run
that code itself, with `eval`.

Most people never type this. `nvmimic self install` puts an `nvm.sh` into
your nvm folder that runs it for you, and your startup file loads that
`nvm.sh` the way it always did.

You can also write the line into your startup file yourself, with the name
of the shell that file belongs to:

  eval \"$(nvmimic init zsh)\"

The name lets nvmimic do everything in one start of the program. A wrong
name does no harm. The code checks which shell is really running it, and in
any other shell it starts nvmimic a second time and gets the same result.
Without a name it always takes that second start.

Loaded this way, the nvm folder is $NVM_DIR if your shell has one, then the
folder `nvmimic self install` set up, then ~/.nvm.";

const USAGE: &str = "\
usage: nvmimic <command>

nvmimic does nvm's job, only faster. In your shell you keep typing `nvm`.
The `nvmimic` command sets nvmimic up, keeps it up to date and removes it
again.

commands:
  self install     set nvmimic up for your nvm folder ($NVM_DIR, usually ~/.nvm)
  self update      get the newest nvmimic
  self rollback    go back to the nvmimic from before the last update
  self eject       put nvm's own scripts back and keep nvmimic for later
  self uninstall   remove nvmimic again
  init [shell]     print the shell code that loads nvmimic, for `eval`
  help             show this text
  --version        print nvmimic's own version

Add --help to any `self` command to read what it does, step by step.";

const INSTALL_USAGE: &str = "\
usage: nvmimic self install [--force] [--dry-run]

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

  --force     don't ask, go ahead
  --dry-run   show what would happen and change nothing

Settings, as environment variables:
  NVM_DIR                the nvm folder, if it isn't ~/.nvm
  NVMIMIC_INSTALL_PATH   where the nvmimic program goes
  NVMIMIC_STATE_DIR, NVMIMIC_CACHE_DIR
                         where nvmimic keeps its other files";

const EJECT_USAGE: &str = "\
usage: nvmimic self eject [--force] [--dry-run]

Hands your nvm folder back to nvm. nvmimic's three scripts leave the folder
and nvm's own come back, so `nvm` in a new shell is nvm again. The nvmimic
program stays installed, and `nvmimic self install` takes over again
whenever you like.

This only works if nvmimic took over from an nvm, because that's when it
kept nvm's scripts. If there was no nvm before nvmimic, it says so and
changes nothing.

It shows what it's about to do and asks first.

  --force     don't ask, go ahead
  --dry-run   show what would happen and change nothing";

const UNINSTALL_USAGE: &str = "\
usage: nvmimic self uninstall [--purge] [--force] [--dry-run]

Removes nvmimic. Your Node versions, aliases and settings stay as they are,
and so does the nvm folder itself.

If nvmimic took over from an nvm, that nvm's scripts come back and it works
as it did before. If there was no nvm before nvmimic, the nvm folder keeps
your Node versions but has no `nvm` command any more.

If you changed one of nvmimic's scripts by hand, your version isn't thrown
away. It's renamed, for example nvm.sh becomes nvm.sh.nvmimic-saved.

It shows what it's about to do and asks first. Where there's no terminal to
ask at, it changes nothing unless you pass --force.

  --purge     also delete the whole nvm folder: every Node version, every
              alias, and nvmimic's backup of nvm's scripts. nvm is not put
              back. The folder and its size are shown before you're asked
  --force     don't ask, go ahead
  --dry-run   show what would happen and change nothing";

const UPDATE_USAGE: &str = "\
usage: nvmimic self update [--force] [--dry-run]

Gets the newest nvmimic and swaps it in. This updates nvmimic only. It has
nothing to do with which nvm version nvmimic copies, or with your Node
versions.

The download is checked before anything changes: its checksum, that
nvmimic's developer signed it, and that it runs. The nvmimic you have now is
kept, and `nvmimic self rollback` brings it back.

If Homebrew or cargo installed nvmimic, this tells you the command to use
instead.

  --force     don't ask, go ahead
  --dry-run   show what would happen and change nothing";

const ROLLBACK_USAGE: &str = "\
usage: nvmimic self rollback [--force] [--dry-run]

Goes back to the nvmimic you had before the last `nvmimic self update`. The
one you leave is kept in its place, so running this twice gets you back
where you started.

  --force     don't ask, go ahead
  --dry-run   show what would happen and change nothing";

/// The help for `nvmimic self <action>`, or for `self` alone.
fn self_usage(action: Option<&str>) -> String {
    match action {
        Some("install") => INSTALL_USAGE.to_string(),
        Some("eject") => EJECT_USAGE.to_string(),
        Some("uninstall") => UNINSTALL_USAGE.to_string(),
        Some("update") => UPDATE_USAGE.to_string(),
        Some("rollback") => ROLLBACK_USAGE.to_string(),
        _ => format!(
            "{INSTALL_USAGE}\n\n{UPDATE_USAGE}\n\n{ROLLBACK_USAGE}\n\n{EJECT_USAGE}\n\n{UNINSTALL_USAGE}"
        ),
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
        ["init"] => return init(None),
        ["init", word] if is_help(word) => {
            println!("{INIT_USAGE}");
            return ExitCode::SUCCESS;
        }
        ["init", shell] => {
            return match Named::parse(shell) {
                Some(named) => init(Some(named)),
                None => {
                    eprintln!(
                        "nvmimic: `init` doesn't know the shell `{shell}`. It knows {}.",
                        Named::NAMES.join(", ")
                    );
                    ExitCode::from(2)
                }
            };
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
            action @ ("install" | "eject" | "uninstall" | "update" | "rollback"),
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
    if let Some(unknown) = flags.iter().find(|flag| {
        !matches!(**flag, "--force" | "--dry-run")
            && !(action == "uninstall" && **flag == "--purge")
    }) {
        eprintln!(
            "nvmimic: `self {action}` has no option `{unknown}`.\n\n{}",
            self_usage(Some(action))
        );
        return ExitCode::from(2);
    }
    let options = nvmimic::install::Options {
        force: flags.contains(&"--force"),
        dry_run: flags.contains(&"--dry-run"),
        purge: flags.contains(&"--purge"),
        terminal: is_tty(0),
    };
    // `self eject` and `self uninstall` say what they would have done
    // before they refuse, so they handle a missing terminal themselves.
    let leaving = matches!(action, "eject" | "uninstall");
    if !leaving && !options.force && !options.dry_run && !options.terminal {
        eprintln!(
            "nvmimic: this asks questions, and there is no terminal to answer them. Run it again with --force."
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
        "eject" => nvmimic::install::self_eject(&env, &options, &mut ui),
        _ => nvmimic::install::self_uninstall(&env, &options, &mut ui),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("nvmimic: {message}");
            match message == nvmimic::install::NO_TERMINAL {
                true => ExitCode::from(2),
                false => ExitCode::FAILURE,
            }
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
