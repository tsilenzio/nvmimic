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
//!
//! Two programs write this record, so a command that changes it holds a
//! lock for as long as it runs ([`lock`]). A second command started in the
//! meantime waits a moment and then says who is in its way.

use crate::dirs::Dirs;
use crate::identity::Identity;
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Seek, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

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

/// Set for a program that is run by a command holding the lock, such as
/// the sister tool's `self install` during a hand-over. That program works
/// under the lock of the command that started it and takes none itself.
pub const LOCK_HELD: &str = "NVMIMIC_SHARED_LOCKED";

/// How long a `self` command waits for another one to finish.
pub const LOCK_WAIT: Duration = Duration::from_secs(2);

/// The lock on the shared record, held until this is dropped.
pub struct Lock {
    held: Option<(fs::File, Dirs)>,
}

impl Drop for Lock {
    fn drop(&mut self) {
        let Some((file, dirs)) = &self.held else {
            return;
        };
        // The file names its holder only while it is held.
        let _ = file.set_len(0);
        // With no record there is no program set up, and nothing is left
        // behind for the lock's sake either.
        if !dirs.record().exists() {
            let _ = fs::remove_file(dirs.shared_lock());
        }
    }
}

/// Takes the lock on the shared record for `identity`'s program, waiting
/// up to `wait` for a command that holds it. `Err` says who holds it.
///
/// The lock is a file beside the shared folder's standard place, so every
/// program finds the same one and the folder can come and go under it.
/// Where the file can't be made or this kind of lock doesn't exist, the
/// command carries on without one.
pub fn lock(dirs: &Dirs, identity: &Identity, wait: Duration) -> Result<Lock, String> {
    let unlocked = Ok(Lock { held: None });
    if std::env::var_os(LOCK_HELD).is_some() {
        return unlocked;
    }
    let path = dirs.shared_lock();
    if let Some(folder) = path.parent() {
        let _ = fs::create_dir_all(folder);
    }
    let started = Instant::now();
    loop {
        let opened = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path);
        let Ok(mut file) = opened else {
            return unlocked;
        };
        // SAFETY: flock only takes the descriptor, which `file` keeps open.
        let taken = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0;
        if taken {
            // A command that finished in between may have removed the file
            // this one opened. Then the lock is on a file nobody else sees.
            let same = match (file.metadata(), fs::metadata(&path)) {
                (Ok(mine), Ok(there)) => mine.ino() == there.ino() && mine.dev() == there.dev(),
                _ => false,
            };
            if !same {
                continue;
            }
            let holder = format!("{} {}\n", identity.name, std::process::id());
            let _ = file.set_len(0);
            let _ = file.write_all(holder.as_bytes());
            return Ok(Lock {
                held: Some((file, dirs.clone())),
            });
        }
        if std::io::Error::last_os_error().raw_os_error() != Some(libc::EWOULDBLOCK) {
            return unlocked;
        }
        if started.elapsed() >= wait {
            let mut holder = String::new();
            let _ = file.rewind();
            let _ = file.read_to_string(&mut holder);
            let who = match holder.trim().split_once(' ') {
                Some((name, process)) => format!(" ({name}, process {process})"),
                None => String::new(),
            };
            return Err(format!(
                "another `self` command is still running{who}, and two of them can't change the \
                 same nvm setup at once. Run this again when it has finished. Nothing was changed."
            ));
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// Tells the sister tool `name` to take over, by running its own `self
/// install --force`. Its scripts and its record are its own to write, so
/// no other program writes them for it.
pub(crate) fn hand_to(name: &str, binary: &Path, nvm_dir: &Path) -> Result<(), String> {
    let output = Command::new(binary)
        .args(["self", "install", "--force"])
        .env("NVM_DIR", nvm_dir)
        .env(LOCK_HELD, "1")
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
    fn one_command_at_a_time_holds_the_lock_and_the_second_says_who_does() {
        let root = std::env::temp_dir().join(format!("nvmimic-lock-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let home = root.to_string_lossy().into_owned();
        let dirs = Dirs::resolve(|name| (name == "HOME").then(|| home.clone())).unwrap();
        let shelfy = Identity {
            name: "shelfy",
            ..Identity::NVMIMIC
        };
        let theirs = Dirs::resolve_for(&shelfy, |name| (name == "HOME").then(|| home.clone()));
        let theirs = theirs.unwrap();
        assert_eq!(dirs.shared_lock(), theirs.shared_lock());

        let first = lock(&dirs, &Identity::NVMIMIC, Duration::ZERO).unwrap();
        let started = Instant::now();
        let second = lock(&theirs, &shelfy, Duration::from_millis(100));
        let said = second.err().unwrap();
        assert!(started.elapsed() >= Duration::from_millis(100));
        let holder = format!("(nvmimic, process {})", std::process::id());
        assert!(said.contains(&holder), "{said}");
        assert!(said.ends_with("Nothing was changed."), "{said}");

        // With no record, nothing is left behind once the lock is let go.
        drop(first);
        assert!(!dirs.shared_lock().exists());
        // With one, the file stays for the next command.
        let record = Record {
            programs: [(
                "nvmimic".to_string(),
                Program {
                    binary: "/h/nvmimic".into(),
                    aside: false,
                },
            )]
            .into(),
            ..Record::default()
        };
        let second = lock(&theirs, &shelfy, Duration::ZERO).unwrap();
        record.write(&dirs).unwrap();
        drop(second);
        assert!(dirs.shared_lock().exists());
        lock(&dirs, &Identity::NVMIMIC, Duration::ZERO).unwrap();
        let _ = fs::remove_dir_all(&root);
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
