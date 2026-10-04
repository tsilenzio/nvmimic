//! Answers nvmimic asks a program for once and then remembers.
//!
//! nvm runs `node --version` and `npm --version` every time it wants to
//! know. Starting a program is the slow part of nvm, so nvmimic keeps each
//! answer in a small file under `$NVM_DIR/.nvmimic/cache/` together with a
//! key: the facts the answer depends on. When the key no longer matches,
//! because the program's file was replaced for example, the program is
//! asked again. Only an answer that came with a clean exit is kept, so a
//! program that cannot run is tried again each time, as nvm tries it.

use crate::context::Nvm;
use std::path::{Path, PathBuf};

/// Size, both timestamps and the inode of a file, as text. Replacing a
/// file changes at least one of them, even when the new copy keeps the old
/// modified time, as files unpacked from an archive do.
pub fn fingerprint(path: &Path) -> Option<String> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::metadata(path).ok()?;
    Some(format!(
        "{} {}.{} {}.{} {}",
        meta.size(),
        meta.mtime(),
        meta.mtime_nsec(),
        meta.ctime(),
        meta.ctime_nsec(),
        meta.ino()
    ))
}

/// FNV-1a, to name a remembered answer after what it belongs to.
fn fnv1a(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// Where one remembered answer lives, and the line that has to match for
/// it to still be true.
pub struct Remembered {
    file: PathBuf,
    key: String,
}

impl Remembered {
    /// `kind` names the question, `subject` what it was asked about, and
    /// `key` everything the answer depends on. `None` when nvmimic is not
    /// installed in this `$NVM_DIR`, where nothing is remembered.
    pub fn new(nvm: &Nvm, kind: &str, subject: &str, key: &[String]) -> Option<Remembered> {
        let own = Path::new(nvm.vars.get("NVM_DIR")).join(".nvmimic");
        if !own.is_dir() {
            return None;
        }
        let name = format!("{:016x}", fnv1a(subject));
        Some(Remembered {
            file: own.join("cache").join(kind).join(name),
            key: key.join("\t").replace('\n', " "),
        })
    }

    pub fn read(&self) -> Option<String> {
        let text = std::fs::read_to_string(&self.file).ok()?;
        let (key, answer) = text.split_once('\n')?;
        (key == self.key).then(|| answer.trim_end_matches('\n').to_string())
    }

    /// Keeps what a program printed, when it exited cleanly and printed one
    /// line. Written beside its final name and renamed into place, so
    /// another shell never reads half a file.
    pub fn keep(&self, output: &std::process::Output, answer: &str) {
        if !output.status.success() || answer.is_empty() || answer.contains('\n') {
            return;
        }
        let Some(dir) = self.file.parent() else {
            return;
        };
        if std::fs::create_dir_all(dir).is_err() {
            return;
        }
        let temporary = self.file.with_extension(std::process::id().to_string());
        if std::fs::write(&temporary, format!("{}\n{answer}\n", self.key)).is_ok()
            && std::fs::rename(&temporary, &self.file).is_err()
        {
            let _ = std::fs::remove_file(&temporary);
        }
    }
}

/// Runs `<program> --version` as the calling shell would, with stderr
/// dropped, and returns what it printed without the trailing line break.
pub fn ask_version(nvm: &Nvm, program: &str) -> Option<(std::process::Output, String)> {
    let mut command = std::process::Command::new(program);
    command
        .arg("--version")
        .env_clear()
        .envs(nvm.vars.exported())
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    if !nvm.vars.get("PWD").is_empty() {
        command.current_dir(nvm.vars.get("PWD"));
    }
    let output = command.output().ok()?;
    let answer = String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_string();
    Some((output, answer))
}
