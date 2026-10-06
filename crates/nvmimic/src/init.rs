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

/// The lines that switch to the default version, behind the test described
/// above. `forwarded` is the list of variables the shell functions hand
/// over on every call, and the test covers exactly those. `None` when the
/// second start is the only safe way.
pub fn load(forwarded: &[&str], shell: Shell, backend: &dyn Backend) -> Option<String> {
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
    Some(format!(
        "if {test}; then\n{}:\nelse\n{SECOND_START}fi\n",
        render(&outcome.effects)
    ))
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
}
