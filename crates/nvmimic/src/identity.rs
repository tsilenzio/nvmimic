//! Who a program built on this library is.
//!
//! nvmimic is one such program. Another one, with its own name, gets its
//! own folders, settings, shell function names and messages from the same
//! code by handing in an [`Identity`]. Almost everything is worked out from
//! the name, so two programs can't end up sharing a folder or a shell
//! variable by accident.
//!
//! | What | For the name `nvmimic` |
//! |---|---|
//! | folders | `~/.local/state/nvmimic` and the others in [`crate::dirs`] |
//! | settings | `NVMIMIC_INSTALL_PATH`, `NVMIMIC_STATE_DIR` and so on |
//! | shell names | `__nvmimic_handoff`, `__NVMIMIC_*`, `_nvmimic_complete` |
//! | files it leaves | `nvm.sh.nvmimic-saved`, `previous/nvmimic-<version>` |
//! | markers in its scripts | `as nvmimic provides it`, `Changed by nvmimic` |
//!
//! Two programs built on this library never share an nvm folder. Each one
//! finds the other's marker in `nvm.sh` and refuses to install over it.

use crate::complete::Command;
use std::borrow::Cow;

/// The names a program built on this library goes by.
#[derive(Debug)]
pub struct Identity {
    /// The program's name: lower-case letters and digits, starting with a
    /// letter. See [`Identity::is_valid`].
    pub name: &'static str,
    /// The program's own version. The shell functions carry it, and a shell
    /// that holds another version's functions is sent new ones.
    pub version: &'static str,
    /// What TAB suggests after the program's own name.
    pub completion: &'static Command,
    /// What package managers call the program, for the commands that
    /// messages tell people to run.
    pub packages: Packages,
    /// Where `self update` gets a newer program. `None` for a program that
    /// doesn't update itself.
    pub release: Option<Release>,
}

#[derive(Debug)]
pub struct Packages {
    /// The Homebrew formula, as in `brew upgrade <homebrew>`.
    pub homebrew: &'static str,
    /// The crate, as in `cargo install <cargo>`.
    pub cargo: &'static str,
}

#[derive(Debug)]
pub struct Release {
    /// The address releases are published under. The newest one's files are
    /// at `<url>/latest/download`.
    pub url: &'static str,
    /// The Apple team that signs releases. `self update` accepts only a
    /// download signed by this team. `None` checks nothing, which is only
    /// right for a build that isn't a release.
    pub team_id: Option<&'static str>,
}

impl Identity {
    /// nvmimic itself. Every entry point that takes no identity means this
    /// one.
    pub const NVMIMIC: Identity = Identity {
        name: "nvmimic",
        version: env!("CARGO_PKG_VERSION"),
        completion: &crate::complete::NVMIMIC,
        packages: Packages {
            homebrew: "nvmimic",
            cargo: "nvmimic-cli",
        },
        release: Some(Release {
            url: crate::update::RELEASES,
            team_id: crate::update::TEAM_ID,
        }),
    };

    /// Whether the name can be part of a shell function name, a variable
    /// name and a file name. `nvm` is taken.
    pub fn is_valid(&self) -> bool {
        let mut letters = self.name.chars();
        letters.next().is_some_and(|c| c.is_ascii_lowercase())
            && letters.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            && self.name != "nvm"
    }

    fn is_nvmimic(&self) -> bool {
        self.name == Identity::NVMIMIC.name
    }

    /// The name in capitals, as settings and shell variables spell it.
    pub fn upper(&self) -> String {
        self.name.to_ascii_uppercase()
    }

    /// The name of one of the program's settings: `setting("STATE_DIR")` is
    /// `NVMIMIC_STATE_DIR` for nvmimic.
    pub fn setting(&self, suffix: &str) -> String {
        format!("{}_{suffix}", self.upper())
    }

    /// The function the shell functions hand their work to.
    pub fn handoff_function(&self) -> String {
        format!("__{}_handoff", self.name)
    }

    /// The first line of this program's `nvm.sh`.
    pub fn marker_line(&self) -> String {
        format!("{MARKER_START}{}{MARKER_END}", self.name)
    }

    /// Text only this program's `nvm.sh` holds.
    pub fn own_marker(&self) -> String {
        format!("as {} provides it", self.name)
    }

    /// Text only this program's `bash_completion` holds.
    pub fn completion_marker(&self) -> String {
        format!("Changed by {}", self.name)
    }

    /// Shell text from this library's `shell/` folder, with this program's
    /// name where nvmimic's is. The licence line keeps naming nvmimic,
    /// which is where the text comes from. For nvmimic nothing changes.
    pub fn shell_text<'a>(&self, text: &'a str) -> Cow<'a, str> {
        if self.is_nvmimic() {
            return Cow::Borrowed(text);
        }
        let upper = self.upper();
        text.split_inclusive('\n')
            .map(|line| match line.contains(LICENCE_LINE) {
                true => line.to_string(),
                false => line
                    .replace("nvmimic", self.name)
                    .replace("NVMIMIC", &upper),
            })
            .collect::<String>()
            .into()
    }
}

const MARKER_START: &str = "# Node Version Manager, as ";
const MARKER_END: &str = " provides it.";
const LICENCE_LINE: &str = "nvmimic is MIT licensed";

/// The name of the program whose `nvm.sh` this is, going by its first
/// line. `None` for official nvm's and for anything else.
pub fn program_of(nvm_sh: &str) -> Option<&str> {
    let name = nvm_sh
        .lines()
        .next()?
        .strip_prefix(MARKER_START)?
        .strip_suffix(MARKER_END)?;
    let plain = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    plain.then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OTHER: Identity = Identity {
        name: "shelfy",
        version: "9.9.9",
        completion: &crate::complete::NVMIMIC,
        packages: Packages {
            homebrew: "shelfy",
            cargo: "shelfy",
        },
        release: None,
    };

    #[test]
    fn nvmimics_shell_text_is_the_files_own_bytes() {
        let text = "__nvmimic_handoff __NVMIMIC_FD\n# nvmimic is MIT licensed: x\n";
        assert!(matches!(
            Identity::NVMIMIC.shell_text(text),
            Cow::Borrowed(same) if same == text
        ));
    }

    #[test]
    fn another_program_gets_its_own_names_and_the_licence_line_stays() {
        let text = "__nvmimic_handoff __NVMIMIC_FD\n# nvmimic is MIT licensed: x\nlast nvmimic";
        assert_eq!(
            OTHER.shell_text(text),
            "__shelfy_handoff __SHELFY_FD\n# nvmimic is MIT licensed: x\nlast shelfy"
        );
        assert_eq!(OTHER.setting("STATE_DIR"), "SHELFY_STATE_DIR");
        assert_eq!(OTHER.handoff_function(), "__shelfy_handoff");
    }

    #[test]
    fn names_have_to_fit_a_shell_function_and_a_file() {
        let named = |name| Identity { name, ..OTHER };
        assert!(Identity::NVMIMIC.is_valid() && OTHER.is_valid());
        for name in ["", "nvm", "Shelfy", "my-tool", "2fast", "a b", "tool_x"] {
            assert!(!named(name).is_valid(), "{name}");
        }
    }

    #[test]
    fn a_marker_line_names_its_program() {
        assert_eq!(
            program_of(&format!("{}\n# more\n", OTHER.marker_line())),
            Some("shelfy")
        );
        assert_eq!(
            program_of("# Node Version Manager, as nvmimic provides it.\n"),
            Some("nvmimic")
        );
        assert_eq!(program_of("# Node Version Manager\n"), None);
        assert_eq!(program_of(""), None);
    }
}
