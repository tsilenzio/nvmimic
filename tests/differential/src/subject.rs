//! The program a run compares with official nvm.
//!
//! nvmimic is one ([`crate::own::Nvmimic`]). Another program built on the
//! nvmimic library is a second: it says how it gets into a sandbox, which
//! shell functions are its own, and where it differs from official nvm on
//! purpose. Recordings of official nvm don't depend on the subject, so a
//! second program replays the same ones.

use std::path::Path;

pub trait Subject: Sync {
    /// The program's name, for labels and sandbox folder names.
    fn name(&self) -> &str;

    /// Puts the program into `nvm_dir` the way its installer would. `side`
    /// is the sandbox: `side/home` is the home folder, and anything of the
    /// program's own belongs in a folder beside it, such as `side/own`,
    /// where the comparison of the home folders doesn't see it. With
    /// `count_launches` each start of the program adds its name as a line to
    /// the file `$NVMIMIC_LAUNCH_LOG` names.
    ///
    /// A program that gets run is linked into the sandbox, never copied:
    /// macOS scans every new file the first time it runs.
    fn install(&self, nvm_dir: &Path, side: &Path, count_launches: bool);

    /// Variables the program needs in the shell that runs a case.
    fn env(&self, _side: &Path) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Shell functions the program defines that official nvm doesn't. They
    /// are left out of the list of functions a run leaves behind.
    fn own_functions(&self) -> Vec<String>;

    /// Rewrites what a run printed or left, after the harness has put
    /// `<home>`, `<install>/` and the like where the sandbox's own paths
    /// were. A program that keeps versions elsewhere maps its folder to
    /// `<install>/` here.
    fn normalize(&self, text: String) -> String {
        text
    }

    /// Whether a file or folder under the home folder is the program's own
    /// and stays out of the comparison. `relative` is its path below home.
    fn ignores_file(&self, _relative: &str) -> bool {
        false
    }

    /// Differences this program has on purpose, see [`Expected`].
    fn expected(&self) -> &Expected {
        const NONE: &Expected = &Expected(Vec::new());
        NONE
    }
}

/// Differences from official nvm that a subject has on purpose, as rules
/// finer than a whole case and shell pair. One rule per line:
///
/// ```text
/// <case id> <shell> <what differs>
/// ```
///
/// Each of the three is a name, or a name ending in `*` for everything that
/// starts with it. What differs is the difference's key: `exit`, `stdout`,
/// `stderr`, `timed out`, or a piece of what the run left, such as
/// `var:NVM_BIN`, `functions`, `node` or `fs:home/.nvm/alias/default`. Text
/// after `#` is a comment.
#[derive(Clone, Debug, Default)]
pub struct Expected(Vec<[String; 3]>);

impl Expected {
    pub fn parse(text: &str) -> Expected {
        let rules = text
            .lines()
            .map(|line| line.split('#').next().unwrap_or("").trim())
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let mut words = line.splitn(3, ' ').map(|word| word.trim().to_string());
                Some([words.next()?, words.next()?, words.next()?])
            })
            .collect();
        Expected(rules)
    }

    /// Whether a rule covers this difference.
    pub fn covers(&self, case: &str, shell: &str, key: &str) -> bool {
        let fits = |pattern: &str, value: &str| match pattern.strip_suffix('*') {
            Some(start) => value.starts_with(start),
            None => pattern == value,
        };
        self.0
            .iter()
            .any(|[c, s, k]| fits(c, case) && fits(s, shell) && fits(k, key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rule_names_a_case_a_shell_and_what_differs() {
        let expected = Expected::parse(
            "# aliases live elsewhere\n\
             INS-* * fs:home/.nvm/alias/*\n\
             VER-03 zsh stdout  # one line\n",
        );
        assert!(expected.covers("INS-12", "bash", "fs:home/.nvm/alias/default"));
        assert!(!expected.covers("INS-12", "bash", "fs:home/.nvm/versions"));
        assert!(!expected.covers("LST-01", "bash", "fs:home/.nvm/alias/default"));
        assert!(expected.covers("VER-03", "zsh", "stdout"));
        assert!(!expected.covers("VER-03", "bash", "stdout"));
        assert!(!expected.covers("VER-03", "zsh", "stderr"));
        assert!(!Expected::default().covers("VER-03", "zsh", "stdout"));
    }
}
