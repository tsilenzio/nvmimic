//! The record that nvmimic and its sister tools share.
//!
//! More than one program built on this library can be installed for the
//! same nvm folder, and only one of them answers `nvm` at a time. This
//! record is how they keep track of each other. It belongs to none of
//! them, so it lives in a folder of its own ([`Dirs::shared_folder`]) and
//! removing one program never takes it away from the others. The last
//! program to be removed takes the folder with it.
//!
//! It holds three things:
//!
//! - official nvm's scripts from before any of the programs took over, as
//!   files in `backup/`, and the lines that describe them;
//! - which programs are set up, and where each one's program file is;
//! - who answers `nvm` now. No name there means official nvm does.
//!
//! A program that was told to step aside, by its own `self eject` or its
//! disable, is marked `aside`. It stays set up, but the other program's
//! uninstall doesn't hand `nvm` to it.

use crate::dirs::Dirs;
use crate::identity::Identity;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// One program that is set up for the nvm folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    /// The program's own file, which its `nvm.sh` runs.
    pub binary: PathBuf,
    /// The program was told to step aside and waits to be told otherwise.
    pub aside: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Record {
    /// The nvm folder the programs are set up for.
    pub nvm_dir: Option<PathBuf>,
    /// The program that answers `nvm`. `None` when official nvm does.
    pub answers: Option<String>,
    /// The programs that are set up, by name.
    pub programs: BTreeMap<String, Program>,
    /// Scripts that were official nvm's, with a copy of each in the backup
    /// folder.
    pub backed_up: Vec<String>,
    /// Scripts that were links when a program first took over, and where
    /// each one pointed. A link goes back only while its target is there.
    pub links: BTreeMap<String, PathBuf>,
    /// Scripts a program added where there were none.
    pub created: Vec<String>,
    /// The shared folder was deleted while a program was installed, and
    /// this record was written again from what that program knew. The
    /// backup that was in the folder is gone.
    pub lost: bool,
}

/// What the note in the shared folder says.
pub const README: &str = "\
This folder belongs to nvmimic and nvmise, two tools that do nvm's job
(https://github.com/tsilenzio/nvmimic). They share it.

  backup/   official nvm's own scripts, from before either tool replaced them
  record    which of the two tools are set up, and which one answers `nvm`

Please don't delete this folder while nvmimic or nvmise is installed. The
backup is how `self uninstall` and `self eject` put official nvm back.

The last of the two to be uninstalled removes this folder.
";

impl Record {
    /// The record as the text of its file.
    pub fn render(&self) -> String {
        let mut text = String::from(
            "# Who answers `nvm`, and official nvm's scripts from before. Programs built on\n\
             # nvmimic's library share this file and write it themselves.\n",
        );
        if let Some(nvm_dir) = &self.nvm_dir {
            text.push_str(&format!("nvm-dir {}\n", nvm_dir.display()));
        }
        if let Some(name) = &self.answers {
            text.push_str(&format!("answers {name}\n"));
        }
        for (name, program) in &self.programs {
            text.push_str(&format!("program {name} {}\n", program.binary.display()));
            if program.aside {
                text.push_str(&format!("aside {name}\n"));
            }
        }
        for name in &self.backed_up {
            text.push_str(&format!("backup {name}\n"));
        }
        for (name, target) in &self.links {
            text.push_str(&format!("link {name} {}\n", target.display()));
        }
        for name in &self.created {
            text.push_str(&format!("created {name}\n"));
        }
        if self.lost {
            text.push_str("lost backup\n");
        }
        text
    }

    /// Reads what [`Record::render`] wrote. A line it doesn't know is
    /// skipped.
    pub fn parse(text: &str) -> Record {
        let mut record = Record::default();
        let mut aside: Vec<&str> = Vec::new();
        for line in text.lines().filter(|line| !line.starts_with('#')) {
            let Some((key, value)) = line.split_once(' ') else {
                continue;
            };
            match key {
                "nvm-dir" => record.nvm_dir = Some(value.into()),
                "answers" => record.answers = Some(value.to_string()),
                "program" => {
                    if let Some((name, binary)) = value.split_once(' ') {
                        record.programs.insert(
                            name.to_string(),
                            Program {
                                binary: binary.into(),
                                aside: false,
                            },
                        );
                    }
                }
                "aside" => aside.push(value),
                "backup" => record.backed_up.push(value.to_string()),
                "link" => {
                    if let Some((name, target)) = value.split_once(' ') {
                        record.links.insert(name.to_string(), target.into());
                    }
                }
                "created" => record.created.push(value.to_string()),
                "lost" => record.lost = true,
                _ => {}
            }
        }
        for name in aside {
            if let Some(program) = record.programs.get_mut(name) {
                program.aside = true;
            }
        }
        record
    }

    /// The shared record, if any program wrote one yet.
    pub fn read(dirs: &Dirs) -> Option<Record> {
        fs::read_to_string(dirs.record())
            .ok()
            .map(|text| Record::parse(&text))
    }

    /// Writes the record, and the note beside it when that is missing. The
    /// file holds the old record or the new one, never part of each. With
    /// no program left in the record, the whole shared folder is removed.
    pub fn write(&self, dirs: &Dirs) -> Result<(), String> {
        let folder = dirs.shared_folder();
        if self.programs.is_empty() {
            let _ = fs::remove_file(dirs.record());
            let _ = fs::remove_file(dirs.shared_readme());
            let _ = fs::remove_dir(dirs.backup());
            let _ = fs::remove_dir(&folder);
            return Ok(());
        }
        fs::create_dir_all(&folder).map_err(|e| format!("{}: {e}", folder.display()))?;
        if !dirs.shared_readme().is_file() {
            crate::install::write_file(&dirs.shared_readme(), README, 0o644)?;
        }
        crate::install::write_file(&dirs.record(), &self.render(), 0o644)
    }

    /// Whether `identity`'s program is the one that answers `nvm`.
    pub fn answers(&self, identity: &Identity) -> bool {
        self.answers.as_deref() == Some(identity.name)
    }

    /// The sister tool that can be handed `nvm`: another program that is
    /// set up, wasn't told to step aside, and still has its program file.
    pub fn sister(&self, identity: &Identity) -> Option<(&str, &Path)> {
        self.programs
            .iter()
            .find(|(name, program)| {
                *name != identity.name && !program.aside && program.binary.is_file()
            })
            .map(|(name, program)| (name.as_str(), program.binary.as_path()))
    }

    /// The other programs that are set up, whatever state they are in.
    pub fn others(&self, identity: &Identity) -> impl Iterator<Item = &str> {
        self.programs
            .keys()
            .map(String::as_str)
            .filter(move |name| *name != identity.name)
    }
}

/// Puts the program's link to the shared folder in place: a link named
/// `shared` in its own state folder. A link that is missing, or points at
/// nothing, is made again and points at the standard place.
pub(crate) fn link(dirs: &Dirs) -> Result<(), String> {
    let link = dirs.shared_link();
    if fs::read_link(&link).is_ok() && fs::metadata(&link).is_ok_and(|meta| meta.is_dir()) {
        return Ok(());
    }
    for dir in [&dirs.state, &dirs.shared] {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let _ = fs::remove_file(&link);
    std::os::unix::fs::symlink(&dirs.shared, &link).map_err(|e| format!("{}: {e}", link.display()))
}

/// Tells the sister tool `name` to take over, by running its own `self
/// install --force`. Its scripts and its record are its own to write, so
/// no other program writes them for it.
pub(crate) fn hand_to(name: &str, binary: &Path, nvm_dir: &Path) -> Result<(), String> {
    let output = Command::new(binary)
        .args(["self", "install", "--force"])
        .env("NVM_DIR", nvm_dir)
        // The program stays where it is, whatever the environment says now.
        .env(
            format!("{}_INSTALL_PATH", name.to_ascii_uppercase()),
            binary,
        )
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("{}: {e}", binary.display()))?;
    if output.status.success() {
        return Ok(());
    }
    let said = String::from_utf8_lossy(&output.stderr);
    Err(said.trim().lines().last().unwrap_or_default().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_record_round_trips() {
        let record = Record {
            nvm_dir: Some("/h/my nvm".into()),
            answers: Some("shelfy".into()),
            programs: [
                (
                    "nvmimic".to_string(),
                    Program {
                        binary: "/h/.local/bin/nvmimic".into(),
                        aside: true,
                    },
                ),
                (
                    "shelfy".to_string(),
                    Program {
                        binary: "/opt/a b/shelfy".into(),
                        aside: false,
                    },
                ),
            ]
            .into(),
            backed_up: vec!["nvm.sh".into(), "nvm-exec".into()],
            links: [("nvm.sh".to_string(), PathBuf::from("/brew/opt/nvm/nvm.sh"))].into(),
            created: vec!["bash_completion".into()],
            lost: true,
        };
        let text = record.render();
        assert_eq!(Record::parse(&text), record);
        assert!(text.contains("program shelfy /opt/a b/shelfy\n"), "{text}");
        assert!(text.contains("aside nvmimic\n"), "{text}");
    }

    #[test]
    fn a_program_that_stepped_aside_or_lost_its_file_is_not_handed_nvm() {
        let here = std::env::current_exe().unwrap();
        let program = |binary: &Path, aside| Program {
            binary: binary.to_path_buf(),
            aside,
        };
        let with = |sister: Program| Record {
            programs: [("shelfy".to_string(), sister)].into(),
            ..Record::default()
        };
        let me = &Identity::NVMIMIC;
        assert_eq!(
            with(program(&here, false)).sister(me),
            Some(("shelfy", here.as_path()))
        );
        assert_eq!(with(program(&here, true)).sister(me), None);
        assert_eq!(
            with(program(Path::new("/nowhere/shelfy"), false)).sister(me),
            None
        );
    }
}
