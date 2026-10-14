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

    /// Called before each test of nvm's own suite, with the sandbox the
    /// test is about to run in. Those sandboxes are used again from test to
    /// test, and the harness starts a new one only when a test left files
    /// in the home folder. A program that keeps versions or aliases beside
    /// the home folder, such as in `side/own`, removes here what an earlier
    /// test left there, so the next test doesn't start with them.
    fn tidy(&self, _side: &Path) {}
}

/// Differences from official nvm that a subject has on purpose, as rules
/// finer than a whole case and shell pair. One rule per line:
///
/// ```text
/// <case id> <shell> <what differs>
/// <case id> <shell> <what differs> ~ <line>
/// ```
///
/// Each part is a name, or a name with `*` at its end, its start or both,
/// for everything that starts with it, ends with it or holds it. What
/// differs is the difference's key: `exit`, `stdout`, `stderr`, `timed
/// out`, or a piece of what the run left, such as `var:NVM_BIN`,
/// `functions`, `node` or `fs:home/.nvm/alias/default`. Text after `#` at
/// the start of a line, or after ` #`, is a comment.
///
/// The first form leaves the whole key out of the comparison. The second
/// takes the lines that fit `<line>` out of that key on both sides and
/// compares what is left, so the rest of what the case prints is still
/// checked. It is for lines official nvm prints that the subject has no
/// counterpart for, such as `Downloading *`.
#[derive(Clone, Debug, Default)]
pub struct Expected(Vec<Rule>);

#[derive(Clone, Debug)]
struct Rule {
    case: String,
    shell: String,
    key: String,
    /// Lines to take out of the key. `None` leaves the whole key out.
    line: Option<String>,
}

fn fits(pattern: &str, value: &str) -> bool {
    let (open_start, rest) = match pattern.strip_prefix('*') {
        Some(rest) if !rest.is_empty() => (true, rest),
        _ => (false, pattern),
    };
    match (open_start, rest.strip_suffix('*')) {
        (false, Some(start)) => value.starts_with(start),
        (true, Some(middle)) => value.contains(middle),
        (true, None) => value.ends_with(rest),
        (false, None) => pattern == value,
    }
}

impl Expected {
    pub fn parse(text: &str) -> Expected {
        let rules = text
            .lines()
            .map(|line| match line.trim_start().starts_with('#') {
                true => "",
                false => line.split(" #").next().unwrap_or("").trim(),
            })
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let mut words = line.splitn(3, ' ').map(str::trim);
                let (case, shell, rest) = (words.next()?, words.next()?, words.next()?);
                let (key, line) = match rest.split_once(" ~ ") {
                    Some((key, line)) => (key.trim(), Some(line.trim().to_string())),
                    None => (rest, None),
                };
                Some(Rule {
                    case: case.to_string(),
                    shell: shell.to_string(),
                    key: key.to_string(),
                    line,
                })
            })
            .collect();
        Expected(rules)
    }

    fn rules<'a>(
        &'a self,
        case: &'a str,
        shell: &'a str,
        key: &'a str,
    ) -> impl Iterator<Item = &'a Rule> {
        self.0.iter().filter(move |rule| {
            fits(&rule.case, case) && fits(&rule.shell, shell) && fits(&rule.key, key)
        })
    }

    /// Whether a rule leaves this whole difference out.
    pub fn covers(&self, case: &str, shell: &str, key: &str) -> bool {
        self.rules(case, shell, key).any(|rule| rule.line.is_none())
    }

    /// `text` without the lines a rule names for this key. `None` when no
    /// rule names a line that is in it.
    pub fn without_lines(&self, case: &str, shell: &str, key: &str, text: &str) -> Option<String> {
        let named: Vec<&str> = self
            .rules(case, shell, key)
            .filter_map(|rule| rule.line.as_deref())
            .collect();
        if named.is_empty() {
            return None;
        }
        let kept: String = text
            .split_inclusive('\n')
            .filter(|line| {
                let line = line.trim_end_matches('\n');
                !named.iter().any(|pattern| fits(pattern, line))
            })
            .collect();
        (kept != text).then_some(kept)
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

    #[test]
    fn a_rule_can_name_lines_and_leave_the_rest_of_the_key_compared() {
        let expected = Expected::parse(
            "INS-* * stderr ~ Downloading *\n\
             INS-* * stderr ~ *Checksums matched!\n\
             INS-* * fs:work/* ~ *checksum with*\n\
             VER-03 zsh stdout ~ exactly this  # and a comment\n",
        );
        // A rule that names lines doesn't leave the key out.
        assert!(!expected.covers("INS-12", "bash", "stderr"));
        let printed = "Downloading https://x/y...\nNow using node v1\nChecksums matched!\n";
        assert_eq!(
            expected.without_lines("INS-12", "bash", "stderr", printed),
            Some("Now using node v1\n".to_string())
        );
        assert_eq!(
            expected.without_lines(
                "INS-12",
                "sh",
                "fs:work/log",
                "file 644\nComputing checksum with shasum\nok"
            ),
            Some("file 644\nok".to_string())
        );
        // Nothing to take out, another key, another case.
        assert_eq!(
            expected.without_lines("INS-12", "bash", "stderr", "all fine\n"),
            None
        );
        assert_eq!(
            expected.without_lines("INS-12", "bash", "stdout", printed),
            None
        );
        assert_eq!(
            expected.without_lines("LST-01", "bash", "stderr", printed),
            None
        );
        assert_eq!(
            expected.without_lines(
                "VER-03",
                "zsh",
                "stdout",
                "exactly this\nexactly this too\n"
            ),
            Some("exactly this too\n".to_string())
        );
    }
}
