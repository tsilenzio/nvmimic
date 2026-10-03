//! Keeps a parity run from overloading macOS's Gatekeeper.
//!
//! macOS scans every new file the first time it is run. The scan blocks the
//! program from starting, takes about 50ms, and asks one of Apple's servers
//! a question on the way. The scans happen one at a time in a system service
//! called `syspolicyd`. A parity run starts thousands of programs a minute,
//! and an earlier run that also created thousands of new scripts put
//! that service so far behind that it crashed over and over, and the whole
//! machine stopped being able to start programs.
//!
//! The harness now avoids running freshly created files where it can, and
//! never deletes a sandbox while a run is going. This module is the backstop
//! for whatever is left. Every two seconds it runs one brand new script and
//! times it. If that takes long, the scanner has fallen behind, and every
//! worker waits until it has caught up. If it stays behind, or `syspolicyd`
//! gets restarted, the run stops.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// How often the probe script runs.
const INTERVAL: Duration = Duration::from_secs(2);
/// A probe slower than this means the scanner is behind, and work pauses.
const SLOW: Duration = Duration::from_millis(500);
/// Work resumes after two probes in a row faster than this.
const RECOVERED: Duration = Duration::from_millis(200);
/// A pause longer than this stops the run.
const GIVE_UP: Duration = Duration::from_secs(180);

pub struct Guard {
    paused: AtomicBool,
    finished: AtomicBool,
    stopped: Mutex<Option<String>>,
    pauses: AtomicU64,
    slowest_ms: AtomicU64,
}

impl Guard {
    /// Starts watching. Probe scripts are written under `dir` and left there.
    pub fn start(dir: &Path) -> Arc<Guard> {
        let guard = Arc::new(Guard {
            paused: AtomicBool::new(false),
            finished: AtomicBool::new(false),
            stopped: Mutex::new(None),
            pauses: AtomicU64::new(0),
            slowest_ms: AtomicU64::new(0),
        });
        let dir = dir.join("probes");
        fs::create_dir_all(&dir).unwrap();
        let watcher = Arc::clone(&guard);
        thread::spawn(move || watcher.watch(&dir));
        guard
    }

    /// Called by a worker before each run. Waits while work is paused, and
    /// returns the reason when the run has been stopped.
    pub fn wait_turn(&self) -> Result<(), String> {
        loop {
            if let Some(reason) = self.stopped.lock().unwrap().clone() {
                return Err(reason);
            }
            if !self.paused.load(Ordering::SeqCst) {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(250));
        }
    }

    /// Ends the watching and says what it saw.
    pub fn finish(&self) -> String {
        self.finished.store(true, Ordering::SeqCst);
        format!(
            "gatekeeper guard: slowest probe {}ms, paused {} time(s)",
            self.slowest_ms.load(Ordering::SeqCst),
            self.pauses.load(Ordering::SeqCst)
        )
    }

    pub fn stopped(&self) -> Option<String> {
        self.stopped.lock().unwrap().clone()
    }

    fn stop(&self, reason: String) {
        eprintln!("gatekeeper guard: stopping the run: {reason}");
        *self.stopped.lock().unwrap() = Some(reason);
    }

    fn watch(&self, dir: &Path) {
        let scanner = syspolicyd_pid();
        let mut number = 0u64;
        let mut fast_in_a_row = 0;
        let mut paused_since: Option<Instant> = None;
        while !self.finished.load(Ordering::SeqCst) {
            if syspolicyd_pid() != scanner {
                self.stop(
                    "syspolicyd, the macOS service that scans new programs, was restarted \
                     during the run. That is how the outage in an earlier run began. Wait a few \
                     minutes before running anything heavy again."
                        .to_string(),
                );
                return;
            }

            number += 1;
            let took = probe(&dir.join(format!("probe-{number}")));
            let ms = took.as_millis() as u64;
            self.slowest_ms.fetch_max(ms, Ordering::SeqCst);

            if took > SLOW && paused_since.is_none() {
                eprintln!("gatekeeper guard: a new script took {ms}ms to start, pausing");
                self.paused.store(true, Ordering::SeqCst);
                self.pauses.fetch_add(1, Ordering::SeqCst);
                paused_since = Some(Instant::now());
                fast_in_a_row = 0;
            } else if let Some(since) = paused_since {
                fast_in_a_row = if took < RECOVERED {
                    fast_in_a_row + 1
                } else {
                    0
                };
                if fast_in_a_row >= 2 {
                    eprintln!("gatekeeper guard: caught up, resuming");
                    self.paused.store(false, Ordering::SeqCst);
                    paused_since = None;
                } else if since.elapsed() > GIVE_UP {
                    self.stop(format!(
                        "new programs were still slow to start after a {}s pause",
                        GIVE_UP.as_secs()
                    ));
                    return;
                }
            }
            thread::sleep(INTERVAL);
        }
    }
}

/// Writes a script nothing has run before, runs it, and returns how long
/// that took. The file is left in place: deleting a file the scanner may
/// still be looking at is part of what went wrong before.
fn probe(script: &Path) -> Duration {
    if fs::write(script, "#!/bin/sh\nexit 0\n").is_err()
        || fs::set_permissions(script, fs::Permissions::from_mode(0o755)).is_err()
    {
        return Duration::ZERO;
    }
    let started = Instant::now();
    let _ = Command::new(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    started.elapsed()
}

/// The process id of `syspolicyd`, or `None` where there is none, as on Linux.
fn syspolicyd_pid() -> Option<String> {
    let output = Command::new("/usr/bin/pgrep")
        .args(["-x", "syspolicyd"])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!pid.is_empty()).then_some(pid)
}

/// The directory this process keeps shared, reusable files in: scripts that
/// sandboxes link to instead of each getting a fresh copy.
pub fn stage_dir() -> PathBuf {
    static STAGE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    STAGE
        .get_or_init(|| {
            let dir = std::env::temp_dir().join(format!("nvmimic-stage-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            dir
        })
        .clone()
}

/// Removes what earlier runs left in the temp directory, once it is at
/// least half an hour old. Nothing is deleted while a run is using it or
/// shortly after, so the scanner never loses a file it has queued.
pub fn sweep_stale() {
    let parent = std::env::temp_dir();
    let Ok(entries) = fs::read_dir(&parent) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let ours = ["nvmimic-mirror-", "nvmimic-diff-", "nvmimic-stage-"]
            .iter()
            .any(|prefix| name.starts_with(prefix));
        let old = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|time| time.elapsed().ok())
            .is_some_and(|age| age.as_secs() > 1800);
        if ours && old && !owner_is_running(&name) {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}

/// Whether the process whose id is in a directory's name, as in
/// `nvmimic-diff-<pid>-<n>`, is still alive.
fn owner_is_running(name: &str) -> bool {
    let Some(pid) = name
        .split('-')
        .find(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
    else {
        return false;
    };
    Command::new("/bin/kill")
        .args(["-0", pid])
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
