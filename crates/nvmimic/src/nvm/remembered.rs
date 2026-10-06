//! Answers nvmimic asks a program for once and then remembers.
//!
//! nvm runs `node --version` and `npm --version` every time it wants to
//! know. Starting a program is the slow part of nvm, so nvmimic keeps each
//! answer in a small file in nvmimic's cache folder together with a
//! key: the facts the answer depends on. When the key no longer matches,
//! because the program's file was replaced for example, the program is
//! asked again. Only an answer that came with a clean exit is kept, so a
//! program that cannot run is tried again each time, as nvm tries it.
//!
//! The rule: a saved answer is reused only when every input nvmimic knows
//! of is unchanged. When one of them cannot be read, or the file is not
//! the kind of program whose answer stays the same, nothing is saved and
//! the program is asked every time.

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

/// Whether a file is a compiled program with one of these names. A script
/// named `node` is some other tool's stand-in, and so is a compiled program
/// with another name that a link called `node` leads to. Either can answer
/// differently from one directory to the next, so neither is remembered.
/// `resolved` is the path after following links.
pub fn is_compiled(resolved: &Path, names: &[&str]) -> bool {
    use std::io::Read;
    let named = resolved
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| names.contains(&name));
    let mut magic = [0u8; 4];
    named
        && std::fs::File::open(resolved)
            .and_then(|mut file| file.read_exact(&mut magic))
            .is_ok()
        && matches!(
            magic,
            // ELF, then Mach-O for one or several processor types, in
            // either byte order.
            [0x7f, b'E', b'L', b'F']
                | [0xfe, 0xed, 0xfa, 0xce | 0xcf]
                | [0xce | 0xcf, 0xfa, 0xed, 0xfe]
                | [0xca, 0xfe, 0xba, 0xbe | 0xbf]
                | [0xbe | 0xbf, 0xba, 0xfe, 0xca]
        )
}

/// What identifies the running operating system build: the kernel's name,
/// release and build text, and on macOS the system's build number too.
/// After a system update every remembered answer is asked for again.
/// `None` when the system does not say.
pub fn system_build() -> Option<String> {
    fn text(field: &[libc::c_char]) -> String {
        let bytes: Vec<u8> = field
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8)
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
    // SAFETY: `uname` fills the struct it is given, and a zeroed `utsname`
    // is a valid one to hand it.
    let name = unsafe {
        let mut name: libc::utsname = std::mem::zeroed();
        if libc::uname(&mut name) != 0 {
            return None;
        }
        name
    };
    let build = format!(
        "{} {} {} {}",
        text(&name.sysname),
        text(&name.release),
        text(&name.version),
        text(&name.machine)
    );
    #[cfg(target_os = "macos")]
    let build = {
        let mut buffer = [0 as libc::c_char; 64];
        let mut len = buffer.len();
        // SAFETY: the name is NUL-terminated, and `len` is the size of the
        // buffer the answer is written into.
        let status = unsafe {
            libc::sysctlbyname(
                c"kern.osversion".as_ptr(),
                buffer.as_mut_ptr().cast(),
                &mut len,
                std::ptr::null_mut(),
                0,
            )
        };
        if status != 0 {
            return None;
        }
        format!("{build} {}", text(&buffer))
    };
    Some(build)
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
    /// `key` everything the answer depends on. The operating system build
    /// is added to every key. `None` when the caller gave no cache folder
    /// or the system build is unknown, where nothing is remembered.
    pub fn new(nvm: &Nvm, kind: &str, subject: &str, key: &[String]) -> Option<Remembered> {
        let cache = nvm.cache_dir.as_ref()?;
        Some(Remembered::on_system(
            cache,
            kind,
            subject,
            key,
            &system_build()?,
        ))
    }

    fn on_system(cache: &Path, kind: &str, subject: &str, key: &[String], system: &str) -> Self {
        let name = format!("{:016x}", fnv1a(subject));
        Remembered {
            file: cache.join(kind).join(name),
            key: format!("{}\t{system}", key.join("\t")).replace('\n', " "),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::process::ExitStatusExt;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("nvmimic-remembered-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn clean_exit() -> std::process::Output {
        std::process::Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    #[test]
    fn a_system_update_asks_again() {
        let cache = scratch("system");
        let key = ["/opt/node".to_string(), "1 2.3 4.5 6".to_string()];
        let before = Remembered::on_system(&cache, "node-version", "/opt/node", &key, "24A335");
        before.keep(&clean_exit(), "v22.1.0");
        assert_eq!(before.read().as_deref(), Some("v22.1.0"));

        let after = Remembered::on_system(&cache, "node-version", "/opt/node", &key, "24B83");
        assert_eq!(after.read(), None);
        std::fs::remove_dir_all(cache).unwrap();
    }

    #[test]
    fn this_system_names_its_build() {
        let build = system_build().expect("uname answers on every supported system");
        assert!(!build.trim().is_empty());
        assert_eq!(system_build().as_deref(), Some(build.as_str()));
    }

    #[test]
    fn only_a_compiled_program_named_node_or_iojs_counts() {
        let dir = scratch("compiled");
        let names = ["node", "iojs"];
        // This test program is itself a compiled one.
        let compiled = std::env::current_exe().unwrap();

        let node = dir.join("node");
        std::fs::copy(&compiled, &node).unwrap();
        assert!(is_compiled(&node, &names));
        let iojs = dir.join("iojs");
        std::fs::copy(&compiled, &iojs).unwrap();
        assert!(is_compiled(&iojs, &names));

        // Another tool's program that a link named `node` leads to.
        let shim = dir.join("some-version-manager");
        std::fs::copy(&compiled, &shim).unwrap();
        assert!(!is_compiled(&shim, &names));

        // A script named `node`.
        let scripts = dir.join("scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        let script = scripts.join("node");
        std::fs::write(&script, "#!/bin/sh\necho v22.1.0\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(!is_compiled(&script, &names));

        // Too short to tell, and missing.
        let short = scripts.join("iojs");
        std::fs::write(&short, "#!").unwrap();
        assert!(!is_compiled(&short, &names));
        assert!(!is_compiled(&dir.join("missing/node"), &names));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
