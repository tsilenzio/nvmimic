//! What a program needs to stand behind the shell functions: answering
//! the hand-off, printing what `init` prints, and keeping an open shell's
//! functions up to date.
//!
//! nvmimic's own command line is a caller of this module, and another
//! program built on the library is a second one. A [`Host`] names the
//! program ([`Identity`]) and the place versions are kept ([`Backend`]).
//! Everything here reads the variables the shell functions pass for one
//! start of the program. Their names carry the program's name, so
//! `__NVMIMIC_FD` for nvmimic.

use crate::backend::Backend;
use crate::context::CallerShell;
use crate::identity::Identity;
use crate::init::Named;
use crate::install::{Bound, Env, Manifest};
use crate::shell::{self, Dialect, Vars};
use crate::{Caller, Outcome};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

/// Variables the shell functions hand over because nvm reads them whether
/// or not they are exported. `cargo xtask gen-shell` writes this list into
/// `shell/init.sh`.
pub const FORWARDED: &[&str] = &[
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

/// How the shell functions ask for one of nvm's functions: the program's
/// first argument. Internal: it is in no help, and its name and what it
/// expects can change in any release. `cargo xtask gen-shell` writes it
/// into `shell/init.sh`.
pub const HANDOFF: &str = "__internal-shell-handoff";

/// A program that stands behind the shell functions.
pub struct Host<'a> {
    pub identity: &'static Identity,
    pub backend: &'a dyn Backend,
    /// Shell text of the program's own that every loading shell runs. It
    /// goes into what `init` prints, after the functions and before the
    /// lines that switch to the default version. Empty for none.
    pub init_extra: &'a str,
}

impl Host<'_> {
    /// The value of one of the variables the shell functions pass:
    /// `passed("FD")` reads `__NVMIMIC_FD` for nvmimic.
    fn passed(&self, name: &str) -> Option<OsString> {
        env::var_os(format!("{}{name}", self.prefix()))
    }

    fn passed_text(&self, name: &str) -> String {
        self.passed(name).map(lossy).unwrap_or_default()
    }

    /// What the names of the passed variables start with.
    fn prefix(&self) -> String {
        format!("__{}_", self.identity.upper())
    }

    /// Runs one of nvm's functions for the shell that called, and returns
    /// its exit status. This is what the program does when its first
    /// argument is [`HANDOFF`]: `function` and `args` are the arguments
    /// after it.
    pub fn handoff(&self, function: &str, args: &[String]) -> u8 {
        self.handoff_with(function, args, |_| String::new())
    }

    /// [`Host::handoff`] with shell text of the program's own added to
    /// what the calling shell runs after the call. `extra` sees what the
    /// call left behind.
    pub fn handoff_with(
        &self,
        function: &str,
        args: &[String],
        extra: impl FnOnce(&Outcome) -> String,
    ) -> u8 {
        // nvm's code runs inside the shell, and bash and dash end a shell
        // that writes to a pipe nobody reads, as in `nvm install 20 | head
        // -1`. Rust programs ignore that signal unless told otherwise. zsh
        // carries on, so this does too there.
        if self.dialect() != Dialect::Zsh {
            // SAFETY: sets a signal back to its default before any thread starts.
            unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL) };
        }

        let caller = Caller {
            vars: self.caller_vars(),
            dialect: self.dialect(),
            stdout_is_tty: is_tty(1),
            aliases: self
                .passed_text("ALIASES")
                .split_whitespace()
                .map(str::to_string)
                .collect(),
            command_info: self.passed_text("COMMAND_INFO"),
            shell: CallerShell {
                arg0: self.passed_text("ARG0"),
                script: self.passed_text("SOURCE"),
                interactive: self
                    .passed("OPTS")
                    .is_some_and(|options| lossy(options).contains('i')),
                bash: self.passed_text("BASH").parse().unwrap_or(0),
            },
            direct_output: true,
            cache_dir: self
                .passed("CACHE")
                .filter(|dir| !dir.is_empty())
                .map(Into::into),
            identity: self.identity,
        };
        let mut stdout = std::io::stdout();
        let mut stderr = std::io::stderr();
        let mut fd3 = |text: &str| write_fd(3, text);
        let outcome = crate::run(
            caller,
            self.backend,
            function,
            args,
            &mut stdout,
            &mut stderr,
            &mut fd3,
        );

        if let Ok(fd) = self.passed_text("FD").parse::<i32>() {
            let text = self.refresh() + &shell::render(&outcome.effects) + &extra(&outcome);
            let _ = write_fd(fd, &text);
        }
        outcome.status.rem_euclid(256) as u8
    }

    /// The path the shell functions should run this program by. `nvm.sh`
    /// and the functions say which path they used, and that is the one to
    /// keep: a path a package manager links to the newest version stays
    /// right after an upgrade. Typed as `<program> init`, it is wherever
    /// the program was found.
    pub fn program(&self) -> Option<PathBuf> {
        let absolute = |path: OsString| Some(PathBuf::from(path)).filter(|path| path.is_absolute());
        self.passed("PROGRAM")
            .and_then(absolute)
            .or_else(|| env::args_os().next().and_then(absolute))
            .or_else(|| env::current_exe().ok())
    }

    /// New shell functions for a shell that holds another version's, which
    /// happens when the program was updated while the shell was open.
    /// Empty when the shell is up to date. The shell runs it with the rest
    /// of what a call sends back. The cache folder stays the one the shell
    /// started with.
    pub fn refresh(&self) -> String {
        let held = self.passed_text("CODE");
        if held.is_empty() || held == self.identity.version {
            return String::new();
        }
        let Some(binary) = self.program() else {
            return String::new();
        };
        let cache = self
            .passed("CACHE")
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from);
        crate::install::functions_text(
            self.identity,
            &Bound {
                binary: &binary,
                cache: cache.as_deref(),
            },
        )
    }

    /// What `<program> init [shell]` prints: the shell code that loads the
    /// program. `Err` when the program can't tell where it is.
    pub fn init(&self, named: Option<Named>) -> Result<String, String> {
        let app = self.identity.name;
        let binary = self
            .program()
            .ok_or(format!("could not work out where the {app} program is."))?;
        // An installation remembers where its cache folder is. Without one
        // the folder comes from the environment, as it would for `self
        // install`.
        let dirs = crate::dirs::Dirs::from_process_for(self.identity);
        let manifest = dirs.as_ref().and_then(Manifest::read);
        let cache = dirs.as_ref().map(|dirs| {
            manifest
                .as_ref()
                .and_then(|manifest| manifest.cache.clone())
                .unwrap_or_else(|| dirs.cache.clone())
        });
        // `nvm.sh` describes the shell it runs `init` for. Typed directly,
        // nothing does, and there is no `nvm.sh` whose folder could be the
        // nvm folder either. That folder is then the one the program was
        // set up for, or the one nvm's installer would pick:
        // `$XDG_CONFIG_HOME/nvm` when that variable is set, and `~/.nvm`
        // when it isn't.
        let described = self.init_shell(cache.clone());
        let nvm_dir = match described {
            Some(_) => None,
            None => manifest.and_then(|manifest| manifest.nvm_dir).or_else(|| {
                let env = Env::from_process_for(self.identity);
                let known = env.xdg_config_home.is_some() || !env.home.as_os_str().is_empty();
                known.then(|| env.nvm_dir())
            }),
        };
        let load = match (described, named) {
            (Some(shell), _) => crate::init::load(FORWARDED, shell, self.backend),
            (None, Some(named)) => {
                let mut seen = self.exported();
                if let Some(nvm_dir) = &nvm_dir
                    && seen.get("NVM_DIR").is_none_or(String::is_empty)
                {
                    seen.insert("NVM_DIR".to_string(), nvm_dir.display().to_string());
                }
                crate::init::load_named(
                    FORWARDED,
                    named,
                    &seen,
                    cache.as_deref(),
                    self.identity,
                    self.backend,
                )
            }
            (None, None) => None,
        };
        Ok(crate::install::init_text(
            self.identity,
            &Bound {
                binary: &binary,
                cache: cache.as_deref(),
            },
            nvm_dir.as_deref(),
            load.as_deref(),
            self.init_extra,
        ))
    }

    /// The exported variables, without the ones `nvm.sh` and the shell
    /// functions pass for one start of the program.
    fn exported(&self) -> BTreeMap<String, String> {
        let prefix = self.prefix();
        env::vars_os()
            .map(|(name, value)| (lossy(name), lossy(value)))
            .filter(|(name, _)| !name.starts_with(&prefix))
            .collect()
    }

    /// The shell `nvm.sh` says it is running `init` for. `None` when `init`
    /// was typed directly.
    fn init_shell(&self, cache_dir: Option<PathBuf>) -> Option<crate::init::Shell> {
        self.passed("OPTS")?;
        let bash: u32 = self.passed_text("BASH").parse().unwrap_or(0);
        let dialect = if !self.passed_text("ZSH").is_empty() {
            Dialect::Zsh
        } else if bash == 0 {
            Dialect::Posix
        } else if self
            .passed_text("SHELLOPTS")
            .split(':')
            .any(|o| o == "posix")
        {
            Dialect::BashPosix
        } else {
            Dialect::Bash
        };
        Some(crate::init::Shell {
            env: self.exported(),
            dialect,
            caller: CallerShell {
                arg0: self.passed_text("ARG0"),
                script: self.passed_text("SOURCE"),
                interactive: self.passed_text("OPTS").contains('i'),
                bash,
            },
            cache_dir,
            identity: self.identity,
        })
    }

    /// The shell's variables: everything exported, plus the forwarded ones
    /// the shell set without exporting.
    fn caller_vars(&self) -> Vars {
        let prefix = self.prefix();
        let mut values: BTreeMap<String, String> = BTreeMap::new();
        let mut exported: BTreeSet<String> = BTreeSet::new();
        for (name, value) in env::vars_os() {
            let name = lossy(name);
            if name.starts_with(&prefix) {
                continue;
            }
            exported.insert(name.clone());
            values.insert(name, lossy(value));
        }

        if let Some(set) = self.passed("SET") {
            let set = lossy(set);
            let set: BTreeSet<&str> = set.split_whitespace().collect();
            for name in FORWARDED {
                if set.contains(name) {
                    values.insert(name.to_string(), self.passed_text(&format!("V_{name}")));
                } else {
                    values.remove(*name);
                    exported.remove(*name);
                }
            }
        }
        Vars::new(values, exported)
    }

    fn dialect(&self) -> Dialect {
        match self.passed_text("SHELL").as_str() {
            "zsh" => Dialect::Zsh,
            "bash" => Dialect::Bash,
            "bashposix" => Dialect::BashPosix,
            _ => Dialect::Posix,
        }
    }
}

fn lossy(value: OsString) -> String {
    value.to_string_lossy().into_owned()
}

/// Whether a descriptor of this process is a terminal.
pub fn is_tty(fd: i32) -> bool {
    // SAFETY: isatty only inspects the descriptor.
    unsafe { libc::isatty(fd) == 1 }
}

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
