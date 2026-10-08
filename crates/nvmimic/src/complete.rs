//! Tab completion: what to suggest for a half-typed `nvm` or `nvmimic`
//! command line.
//!
//! Everything comes from one table, [`NVM`] for the `nvm` command and
//! [`NVMIMIC`] for nvmimic's own. Each command, flag and argument in it says
//! what it [`Needs`] from the place versions are kept. A [`Backend`] that
//! can't do something, building from source for example, answers `false` in
//! [`Backend::offers`], and those entries are left out. Another program
//! built on this library gets its completion from the same table that way.
//!
//! Nothing here goes to the network. Versions that could be installed are
//! not listed, because finding them out means a download on every TAB.
//!
//! [`Backend`]: crate::backend::Backend
//! [`Backend::offers`]: crate::backend::Backend::offers

/// What an entry needs from the place versions are kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Needs {
    /// Works with any store.
    Nothing,
    /// Asks nodejs.org or a mirror what exists.
    Remote,
    /// Downloads a release and unpacks it.
    Download,
    /// Builds Node from its source code.
    Source,
    /// Installs npm packages into a version.
    Packages,
    /// Keeps downloads in nvm's cache folder.
    Cache,
    /// Removes an installed version.
    Removal,
}

/// What one argument of a command takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Takes {
    /// An installed version, or an alias that names one.
    Version,
    /// The name of an alias.
    Alias,
    /// A version that may not be installed yet. Only the names nvm always
    /// knows are suggested, never a list from the network.
    AnyVersion,
    /// Something nvmimic has no list for, such as a program to run.
    Anything,
}

#[derive(Debug)]
pub struct Flag {
    pub name: &'static str,
    pub about: &'static str,
    pub needs: Needs,
}

#[derive(Debug)]
pub struct Command {
    pub name: &'static str,
    pub about: &'static str,
    pub needs: Needs,
    pub flags: &'static [Flag],
    /// What each argument after the command takes, in order.
    pub takes: &'static [Takes],
    /// Commands that follow this one, as `dir` follows `nvm cache`.
    pub commands: &'static [Command],
}

/// One line of the answer: the word, and a few words about it for shells
/// that show them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion {
    pub word: String,
    pub about: &'static str,
}

/// A flag that works with any store.
pub const fn flag(name: &'static str, about: &'static str) -> Flag {
    Flag {
        name,
        about,
        needs: Needs::Nothing,
    }
}

/// A command with no commands of its own after it.
pub const fn command(
    name: &'static str,
    about: &'static str,
    needs: Needs,
    flags: &'static [Flag],
    takes: &'static [Takes],
) -> Command {
    Command {
        name,
        about,
        needs,
        flags,
        takes,
        commands: &[],
    }
}

const SILENT: Flag = flag("--silent", "print nothing");
const LTS: Flag = flag("--lts", "the newest long-term support version");
const LTS_NAME: Flag = flag("--lts=", "the newest version of one LTS line");
const NO_COLORS: Flag = flag("--no-colors", "no colors in the output");

const INSTALL_FLAGS: &[Flag] = &[
    Flag {
        name: "-s",
        about: "build from source, skip the ready-made download",
        needs: Needs::Source,
    },
    flag("-b", "ready-made download only, never build"),
    Flag {
        name: "--reinstall-packages-from=",
        about: "also install the npm packages another version has",
        needs: Needs::Packages,
    },
    LTS,
    LTS_NAME,
    Flag {
        name: "--skip-default-packages",
        about: "leave out the packages listed in default-packages",
        needs: Needs::Packages,
    },
    Flag {
        name: "--latest-npm",
        about: "then upgrade npm to the newest that works",
        needs: Needs::Packages,
    },
    flag("--no-progress", "no progress bar"),
    Flag {
        name: "--offline",
        about: "install from the cache, download nothing",
        needs: Needs::Cache,
    },
    flag("--alias=", "then point an alias at the version"),
    flag("--default", "then make the version the default"),
    flag("--save", "then write the version to .nvmrc"),
];
const USE_FLAGS: &[Flag] = &[
    SILENT,
    LTS,
    LTS_NAME,
    flag("--save", "write the version to .nvmrc"),
];
const RUN_FLAGS: &[Flag] = &[SILENT, LTS, LTS_NAME];
const LS_FLAGS: &[Flag] = &[NO_COLORS, flag("--no-alias", "leave the aliases out")];
const REMOTE_FLAGS: &[Flag] = &[LTS, LTS_NAME, NO_COLORS];

const LS_ABOUT: &str = "list installed versions";
const LS_REMOTE_ABOUT: &str = "list versions that can be installed";

/// The `nvm` command. The commands up to `which` are in the order nvm's own
/// completion offers them.
pub static NVM: Command = Command {
    name: "nvm",
    about: "Node Version Manager",
    needs: Needs::Nothing,
    flags: &[
        flag("--help", "show nvm's help"),
        flag("--version", "print nvm's version"),
        NO_COLORS,
    ],
    takes: &[],
    commands: &[
        command("help", "show nvm's help", Needs::Nothing, &[], &[]),
        command(
            "install",
            "download and install a version",
            Needs::Download,
            INSTALL_FLAGS,
            &[Takes::AnyVersion],
        ),
        command(
            "uninstall",
            "remove an installed version",
            Needs::Removal,
            &[LTS, LTS_NAME],
            &[Takes::Version],
        ),
        command(
            "use",
            "switch this shell to a version",
            Needs::Nothing,
            USE_FLAGS,
            &[Takes::Version],
        ),
        command(
            "run",
            "run node from a version",
            Needs::Nothing,
            RUN_FLAGS,
            &[Takes::Version, Takes::Anything],
        ),
        command(
            "exec",
            "run a command with a version",
            Needs::Nothing,
            RUN_FLAGS,
            &[Takes::Version, Takes::Anything],
        ),
        command(
            "alias",
            "show aliases, or set one",
            Needs::Nothing,
            &[NO_COLORS],
            &[Takes::Alias, Takes::Version],
        ),
        command(
            "unalias",
            "delete an alias",
            Needs::Nothing,
            &[],
            &[Takes::Alias],
        ),
        command(
            "reinstall-packages",
            "install the npm packages another version has",
            Needs::Packages,
            &[],
            &[Takes::Version],
        ),
        command(
            "current",
            "show the version in use",
            Needs::Nothing,
            &[],
            &[],
        ),
        command(
            "list",
            LS_ABOUT,
            Needs::Nothing,
            LS_FLAGS,
            &[Takes::Version],
        ),
        command("ls", LS_ABOUT, Needs::Nothing, LS_FLAGS, &[Takes::Version]),
        command(
            "list-remote",
            LS_REMOTE_ABOUT,
            Needs::Remote,
            REMOTE_FLAGS,
            &[Takes::Anything],
        ),
        command(
            "ls-remote",
            LS_REMOTE_ABOUT,
            Needs::Remote,
            REMOTE_FLAGS,
            &[Takes::Anything],
        ),
        command(
            "install-latest-npm",
            "upgrade npm in the version in use",
            Needs::Packages,
            &[],
            &[],
        ),
        Command {
            name: "cache",
            about: "show or empty the download cache",
            needs: Needs::Cache,
            flags: &[],
            takes: &[],
            commands: &[
                command("dir", "print the cache folder", Needs::Cache, &[], &[]),
                command("clear", "empty the cache folder", Needs::Cache, &[], &[]),
            ],
        },
        command(
            "deactivate",
            "take nvm's version off PATH",
            Needs::Nothing,
            &[SILENT],
            &[],
        ),
        command(
            "unload",
            "remove nvm from this shell",
            Needs::Nothing,
            &[],
            &[],
        ),
        command(
            "version",
            "resolve a description to an installed version",
            Needs::Nothing,
            &[],
            &[Takes::Version],
        ),
        command(
            "version-remote",
            "resolve a description to a version that can be installed",
            Needs::Remote,
            &[LTS, LTS_NAME],
            &[Takes::AnyVersion],
        ),
        command(
            "which",
            "print the path of a version's node",
            Needs::Nothing,
            &[SILENT],
            &[Takes::Version],
        ),
        command(
            "set-colors",
            "set the five colors of the version list",
            Needs::Nothing,
            &[],
            &[Takes::Anything],
        ),
    ],
};

const FORCE: Flag = flag("--force", "don't ask, go ahead");
const DRY_RUN: Flag = flag("--dry-run", "show what would happen, change nothing");
const SELF_FLAGS: &[Flag] = &[FORCE, DRY_RUN];

/// nvmimic's own command.
pub static NVMIMIC: Command = Command {
    name: "nvmimic",
    about: "nvm's job, only faster",
    needs: Needs::Nothing,
    flags: &[
        flag("--help", "show nvmimic's help"),
        flag("--version", "print nvmimic's version"),
    ],
    takes: &[],
    commands: &[
        Command {
            name: "self",
            about: "set nvmimic up, update it or remove it",
            needs: Needs::Nothing,
            flags: &[flag("--help", "what each of these does")],
            takes: &[],
            commands: &[
                command(
                    "install",
                    "set nvmimic up for your nvm folder",
                    Needs::Nothing,
                    SELF_FLAGS,
                    &[],
                ),
                command(
                    "update",
                    "get the newest nvmimic",
                    Needs::Nothing,
                    SELF_FLAGS,
                    &[],
                ),
                command(
                    "rollback",
                    "go back to the nvmimic from before the last update",
                    Needs::Nothing,
                    SELF_FLAGS,
                    &[],
                ),
                command(
                    "eject",
                    "put nvm's own scripts back, keep nvmimic",
                    Needs::Nothing,
                    SELF_FLAGS,
                    &[],
                ),
                command(
                    "uninstall",
                    "remove nvmimic",
                    Needs::Nothing,
                    &[
                        flag("--purge", "also delete the whole nvm folder"),
                        FORCE,
                        DRY_RUN,
                    ],
                    &[],
                ),
            ],
        },
        Command {
            name: "init",
            about: "print the shell code that loads nvmimic",
            needs: Needs::Nothing,
            flags: &[flag("--help", "what init is for")],
            takes: &[],
            // Keep in step with `crate::init::Named`.
            commands: &[
                command("bash", "for bash", Needs::Nothing, &[], &[]),
                command("zsh", "for zsh", Needs::Nothing, &[], &[]),
                command("sh", "for sh", Needs::Nothing, &[], &[]),
                command("dash", "for dash", Needs::Nothing, &[], &[]),
            ],
        },
        command("help", "show nvmimic's help", Needs::Nothing, &[], &[]),
    ],
};

/// The suggestions for the word at `words[current]`. `words[0]` is the
/// command itself. `offers` says whether the store can do what an entry
/// needs, and `list` gives the words an argument can take.
pub fn suggest(
    root: &'static Command,
    words: &[String],
    current: usize,
    offers: &dyn Fn(Needs) -> bool,
    list: &mut dyn FnMut(Takes) -> Vec<String>,
) -> Vec<Suggestion> {
    let typed = words.get(current).map_or("", String::as_str);
    // Walk the words before the one being typed: which command are we in,
    // and how many of its arguments are already there.
    let mut command = root;
    let mut arguments = 0;
    for word in words.iter().take(current).skip(1) {
        if word.starts_with('-') {
            continue;
        }
        let next = command
            .commands
            .iter()
            .find(|next| arguments == 0 && next.name == word && offers(next.needs));
        match next {
            Some(next) => command = next,
            None => arguments += 1,
        }
    }

    let about = |about: &'static str| move |word: String| Suggestion { word, about };
    let found: Vec<Suggestion> = if typed.starts_with('-') {
        command
            .flags
            .iter()
            .filter(|flag| offers(flag.needs))
            .map(|flag| Suggestion {
                word: flag.name.to_string(),
                about: flag.about,
            })
            .collect()
    } else if arguments == 0 && !command.commands.is_empty() {
        command
            .commands
            .iter()
            .filter(|next| offers(next.needs))
            .map(|next| Suggestion {
                word: next.name.to_string(),
                about: next.about,
            })
            .collect()
    } else {
        match command.takes.get(arguments) {
            Some(Takes::Anything) | None => Vec::new(),
            // No note beside these: zsh puts every entry with the same
            // note on one row, which made a single row of all the names.
            Some(takes) => list(*takes).into_iter().map(about("")).collect(),
        }
    };
    found
        .into_iter()
        .filter(|suggestion| suggestion.word.starts_with(typed))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(line: &str) -> Vec<String> {
        line.split(' ').map(str::to_string).collect()
    }

    fn ask(root: &'static Command, line: &str, offers: &dyn Fn(Needs) -> bool) -> Vec<String> {
        let words = words(line);
        let mut list = |takes: Takes| match takes {
            Takes::Alias => vec!["default".to_string(), "mine".to_string()],
            Takes::Version => vec!["v20.1.0".to_string(), "v22.3.0".to_string()],
            _ => vec!["node".to_string()],
        };
        suggest(root, &words, words.len() - 1, offers, &mut list)
            .into_iter()
            .map(|suggestion| suggestion.word)
            .collect()
    }

    #[test]
    fn commands_come_first_in_the_order_nvm_offers_them() {
        let all = ask(&NVM, "nvm ", &|_| true);
        assert_eq!(all[..4], ["help", "install", "uninstall", "use"]);
        assert_eq!(all.last().unwrap(), "set-colors");
        assert_eq!(ask(&NVM, "nvm de", &|_| true), ["deactivate"]);
    }

    #[test]
    fn a_dash_asks_for_the_flags_of_the_command_typed_so_far() {
        assert_eq!(ask(&NVM, "nvm use --s", &|_| true), ["--silent", "--save"]);
        assert_eq!(ask(&NVM, "nvm --v", &|_| true), ["--version"]);
        assert_eq!(
            ask(&NVMIMIC, "nvmimic self uninstall --", &|_| true),
            ["--purge", "--force", "--dry-run"]
        );
    }

    #[test]
    fn arguments_follow_the_table_and_flags_in_between_dont_count() {
        assert_eq!(ask(&NVM, "nvm use --silent v22", &|_| true), ["v22.3.0"]);
        assert_eq!(ask(&NVM, "nvm alias m", &|_| true), ["mine"]);
        assert_eq!(ask(&NVM, "nvm alias mine v20", &|_| true), ["v20.1.0"]);
        assert_eq!(ask(&NVM, "nvm cache ", &|_| true), ["dir", "clear"]);
        assert!(ask(&NVM, "nvm exec v20.1.0 ", &|_| true).is_empty());
        assert_eq!(
            ask(&NVMIMIC, "nvmimic self ", &|_| true),
            ["install", "update", "rollback", "eject", "uninstall"]
        );
    }

    #[test]
    fn what_a_store_cannot_do_is_left_out() {
        let no_source = |needs: Needs| needs != Needs::Source && needs != Needs::Remote;
        assert!(!ask(&NVM, "nvm install -", &no_source).contains(&"-s".to_string()));
        assert!(ask(&NVM, "nvm install -", &|_| true).contains(&"-s".to_string()));
        let commands = ask(&NVM, "nvm ", &no_source);
        assert!(!commands.contains(&"ls-remote".to_string()));
        assert!(commands.contains(&"ls".to_string()));
    }
}
