//! The last lines of what `nvmimic init` prints: switching to the default
//! version without starting the program a second time.
//!
//! nvm switches to the default version when a shell loads it. nvmimic used
//! to do that with a second start of the program, once the shell had the
//! functions. `init` now works the switch out during its one start and
//! prints it.
//!
//! The catch is that a program only sees the variables a shell exported,
//! and nvm reads some whether they are exported or not. So the switch is
//! printed behind a test the shell runs: does every variable `init` relied
//! on hold what `init` saw. When the test fails, the shell does what it did
//! before and starts the program again, this time handing every variable
//! over. `init` prints only that second way whenever it can't be sure of
//! the first.
//!
//! `nvm.sh` tells `init` which shell is reading. Someone who writes
//! `eval "$(nvmimic init zsh)"` into a startup file names the shell
//! instead, and [`load_named`] puts one more test in front: is this really
//! that shell. A wrong name is not an error. The shell starts the program
//! a second time and ends up in the same place.

use crate::backend::Backend;
use crate::context::CallerShell;
use crate::shell::{Dialect, Vars, quote, render};
use crate::{Caller, run};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// What the shell runs when the switch can't be printed: nvm's own last
/// line, which starts the program a second time.
pub const SECOND_START: &str = "nvm_process_parameters \"$@\"\n";

/// The shell `init` is printing for, as `nvm.sh` described it.
pub struct Shell {
    /// The exported variables, which is all a program sees.
    pub env: BTreeMap<String, String>,
    pub dialect: Dialect,
    pub caller: CallerShell,
    pub cache_dir: Option<PathBuf>,
}

/// A shell named on the command line, as in `nvmimic init zsh`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Named {
    Bash,
    Zsh,
    Sh,
    Dash,
}

/// Tests a shell runs to say which shell it is. They read what the shell
/// itself sets and never `$SHELL`, which names the login shell and says
/// nothing about the one reading the text.
const IS_ZSH: &str = "[ -n \"${ZSH_VERSION-}\" ]";
const IS_BASH: &str = "[ -z \"${ZSH_VERSION-}\" ] && [ -n \"${BASH_VERSION-}\" ] &&
  { case \":${SHELLOPTS-}:\" in *:posix:*) false ;; esac; }";
const IS_BASH_POSIX: &str = "[ -z \"${ZSH_VERSION-}\" ] && [ -n \"${BASH_VERSION-}\" ] &&
  { case \":${SHELLOPTS-}:\" in *:posix:*) ;; *) false ;; esac; }";
const IS_PLAIN: &str = "[ -z \"${ZSH_VERSION-}${BASH_VERSION-}\" ]";

impl Named {
    /// The names `nvmimic init` takes.
    pub const NAMES: &'static [&'static str] = &["bash", "zsh", "sh", "dash"];

    pub fn parse(name: &str) -> Option<Named> {
        match name {
            "bash" => Some(Named::Bash),
            "zsh" => Some(Named::Zsh),
            "sh" => Some(Named::Sh),
            "dash" => Some(Named::Dash),
            _ => None,
        }
    }

    /// The shells the name can mean, each with the test that tells it
    /// apart. `sh` is bash in POSIX mode on macOS and dash on most Linux
    /// systems, so it gets both.
    fn shells(self) -> &'static [(Dialect, &'static str)] {
        match self {
            Named::Bash => &[(Dialect::Bash, IS_BASH)],
            Named::Zsh => &[(Dialect::Zsh, IS_ZSH)],
            Named::Sh => &[
                (Dialect::BashPosix, IS_BASH_POSIX),
                (Dialect::Posix, IS_PLAIN),
            ],
            Named::Dash => &[(Dialect::Posix, IS_PLAIN)],
        }
    }
}

/// The lines that switch to the default version, behind the test described
/// above. `forwarded` is the list of variables the shell functions hand
/// over on every call, and the test covers exactly those. `None` when the
/// second start is the only safe way.
pub fn load(forwarded: &[&str], shell: Shell, backend: &dyn Backend) -> Option<String> {
    let Switch { test, changes } = switch(forwarded, shell, backend)?;
    Some(format!(
        "if {test}; then\n{changes}:\nelse\n{SECOND_START}fi\n"
    ))
}

/// [`load`] for a shell that was named and not described. `env` is what
/// the program can see: the exported variables. The switch for each shell
/// the name can mean is printed behind a test for that shell, and any
/// other shell takes the second start.
pub fn load_named(
    forwarded: &[&str],
    named: Named,
    env: &BTreeMap<String, String>,
    cache_dir: Option<&std::path::Path>,
    backend: &dyn Backend,
) -> Option<String> {
    let mut text = String::new();
    for (dialect, is_shell) in named.shells() {
        // Nothing says which bash this is, and bash 3 reads an alias file
        // with a stray NUL byte in it differently from later ones. When
        // the two would not end up the same, neither is printed.
        let majors: &[u32] = if dialect.is_bash() { &[3, 4] } else { &[0] };
        let mut switches = majors.iter().map(|&bash| {
            let shell = Shell {
                env: env.clone(),
                dialect: *dialect,
                caller: CallerShell {
                    bash,
                    ..CallerShell::default()
                },
                cache_dir: cache_dir.map(PathBuf::from),
            };
            switch(forwarded, shell, backend)
        });
        let Some(Some(first)) = switches.next() else {
            continue;
        };
        if switches.any(|other| other.as_ref() != Some(&first)) {
            continue;
        }
        let Switch { test, changes } = first;
        let keyword = if text.is_empty() { "if" } else { "elif" };
        text.push_str(&format!(
            "{keyword} {is_shell} &&\n  {test}; then\n{changes}:\n"
        ));
    }
    if text.is_empty() {
        return None;
    }
    Some(format!("{text}else\n{SECOND_START}fi\n"))
}

/// The switch to the default version for one shell.
#[derive(PartialEq, Eq)]
struct Switch {
    /// True in a shell whose variables hold what `init` saw.
    test: String,
    /// What the switch changes in the shell, as shell code.
    changes: String,
}

fn switch(forwarded: &[&str], shell: Shell, backend: &dyn Backend) -> Option<Switch> {
    let Shell {
        mut env,
        dialect,
        caller,
        cache_dir,
    } = shell;

    // Loading works `NVM_DIR` out when it's missing and repairs it when it
    // ends in a slash. Both happen in the shell, after `init` has run.
    let nvm_dir = env.get("NVM_DIR").filter(|dir| !dir.is_empty())?;
    if nvm_dir.ends_with('/') && nvm_dir.contains(|c| c != '/') {
        return None;
    }
    // With this set, `nvm use` also moves a link on disk.
    if env
        .get("NVM_SYMLINK_CURRENT")
        .is_some_and(|v| !v.is_empty())
    {
        return None;
    }
    // The shell sets this before it gets to the switch.
    if env.get("NVM_CD_FLAGS").is_none_or(|flags| flags.is_empty()) {
        env.insert("NVM_CD_FLAGS".to_string(), String::new());
    }
    if dialect.is_zsh() {
        env.insert("NVM_CD_FLAGS".to_string(), "-q".to_string());
    }
    // zsh ties `MANPATH` to its `manpath` array, so there it is always set,
    // to nothing when nobody exported it.
    let mut exported: BTreeSet<String> = env.keys().cloned().collect();
    if dialect.is_zsh() && !env.contains_key("MANPATH") {
        env.insert("MANPATH".to_string(), String::new());
        exported.remove("MANPATH");
    }

    let seen: Vec<(&str, Option<String>)> = forwarded
        .iter()
        .map(|name| (*name, env.get(*name).cloned()))
        .collect();

    let (mut out, mut err) = (Vec::new(), Vec::new());
    let mut wrote_fd3 = false;
    let outcome = run(
        Caller {
            vars: Vars::new(env, exported),
            dialect,
            stdout_is_tty: false,
            aliases: Vec::new(),
            command_info: String::new(),
            shell: caller,
            direct_output: false,
            cache_dir,
        },
        backend,
        "nvm_auto",
        &["use".to_string()],
        &mut out,
        &mut err,
        &mut |_| {
            wrote_fd3 = true;
            true
        },
    );
    // `init`'s output is the code the shell reads, so a switch that prints
    // anything has to run in the shell. A failed one does too, since its
    // status is what loading returns.
    if outcome.status != 0 || !out.is_empty() || !err.is_empty() || wrote_fd3 {
        return None;
    }

    // Arguments (`--no-use`, `--install`) always take the second way.
    let mut test = String::from("[ \"$#\" -eq 0 ]");
    let unset: String = seen
        .iter()
        .filter(|(_, value)| value.is_none())
        .map(|(name, _)| format!("${{{name}+x}}"))
        .collect();
    if !unset.is_empty() {
        test.push_str(&format!(" &&\n  [ -z \"{unset}\" ]"));
    }
    for (name, value) in &seen {
        if let Some(value) = value {
            // The `x` tells a variable set to nothing from one that isn't set.
            test.push_str(&format!(
                " &&\n  [ \"${{{name}+x}}${{{name}-}}\" = {} ]",
                quote(&format!("x{value}"))
            ));
        }
    }
    Some(Switch {
        test,
        changes: render(&outcome.effects),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::NvmDirBackend;

    fn shell(vars: &[(&str, &str)]) -> Shell {
        Shell {
            env: vars
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
            dialect: Dialect::Bash,
            caller: CallerShell::default(),
            cache_dir: None,
        }
    }

    #[test]
    fn the_second_start_is_all_there_is_without_a_usable_nvm_dir() {
        let forwarded = ["NVM_DIR", "PATH"];
        for vars in [
            vec![("PATH", "/usr/bin")],
            vec![("NVM_DIR", ""), ("PATH", "/usr/bin")],
            vec![("NVM_DIR", "/nowhere/.nvm/"), ("PATH", "/usr/bin")],
            vec![
                ("NVM_DIR", "/nowhere/.nvm"),
                ("NVM_SYMLINK_CURRENT", "true"),
            ],
        ] {
            assert!(load(&forwarded, shell(&vars), &NvmDirBackend).is_none());
        }
    }

    #[test]
    fn the_test_covers_every_forwarded_variable() {
        // No default alias in a folder that doesn't exist: nothing to switch
        // to, which is a quiet success.
        let forwarded = ["NVM_DIR", "PATH", "NVM_CD_FLAGS", "TERM", "PREFIX"];
        let vars = [("NVM_DIR", "/nowhere/.nvm"), ("PATH", "/usr/bin:/bin")];
        let text = load(&forwarded, shell(&vars), &NvmDirBackend).unwrap();
        assert!(text.starts_with("if [ \"$#\" -eq 0 ] &&\n"), "{text}");
        assert!(text.contains("[ -z \"${TERM+x}${PREFIX+x}\" ]"), "{text}");
        assert!(
            text.contains("[ \"${NVM_DIR+x}${NVM_DIR-}\" = 'x/nowhere/.nvm' ]"),
            "{text}"
        );
        assert!(
            text.contains("[ \"${PATH+x}${PATH-}\" = 'x/usr/bin:/bin' ]"),
            "{text}"
        );
        // Loading sets this to nothing when it was unset.
        assert!(
            text.contains("[ \"${NVM_CD_FLAGS+x}${NVM_CD_FLAGS-}\" = 'x' ]"),
            "{text}"
        );
        assert!(
            text.ends_with(&format!(":\nelse\n{SECOND_START}fi\n")),
            "{text}"
        );
    }

    #[test]
    fn a_named_shell_is_tested_for_before_its_switch_is_used() {
        let forwarded = ["NVM_DIR", "PATH"];
        let env: BTreeMap<String, String> = [("NVM_DIR", "/nowhere/.nvm"), ("PATH", "/usr/bin")]
            .into_iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        let text = |named| load_named(&forwarded, named, &env, None, &NvmDirBackend).unwrap();

        let zsh = text(Named::Zsh);
        assert!(
            zsh.starts_with(&format!("if {IS_ZSH} &&\n  [ \"$#\" -eq 0 ]")),
            "{zsh}"
        );
        assert!(!zsh.contains("elif"), "{zsh}");
        assert!(text(Named::Bash).starts_with(&format!("if {IS_BASH} &&\n")));
        assert!(text(Named::Dash).starts_with(&format!("if {IS_PLAIN} &&\n")));

        // `sh` is one of two shells, and each gets its own switch.
        let sh = text(Named::Sh);
        assert!(sh.starts_with(&format!("if {IS_BASH_POSIX} &&\n")), "{sh}");
        assert!(sh.contains(&format!(":\nelif {IS_PLAIN} &&\n")), "{sh}");
        for text in [zsh, sh] {
            assert!(text.ends_with(&format!(":\nelse\n{SECOND_START}fi\n")));
            assert!(!text.contains("$SHELL") && !text.contains("${SHELL-}"));
        }
    }

    #[test]
    fn a_named_shell_gets_the_second_start_alone_where_load_would() {
        let env: BTreeMap<String, String> = [("NVM_DIR".to_string(), String::new())].into();
        for name in Named::NAMES {
            let named = Named::parse(name).unwrap();
            assert!(load_named(&["NVM_DIR"], named, &env, None, &NvmDirBackend).is_none());
        }
        assert_eq!(Named::parse("fish"), None);
    }
}
