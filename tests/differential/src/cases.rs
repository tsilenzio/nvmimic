//! Reference cases: each file in `cases/` runs against real nvm and nvmimic,
//! and the two must agree. A case file starts with header comments:
//!
//! ```text
//! # id: USE-01
//! # shells: bash zsh sh dash      (optional, defaults to every shell present)
//! # source: --no-use              (optional, arguments for `. nvm.sh`)
//! # setup: nvm install 24.15.0    (optional, repeatable, output discarded)
//! # covers: nvm_ls nvm:use        (optional, the nvm functions it exercises)
//! # mirror: plain                 (optional, see below)
//! ```
//!
//! followed by the shell body whose output is compared.
//!
//! Installed versions normally share their `node` and `npm` scripts through
//! symlinks (see [`crate::mirror`]). A case that changes those files needs
//! real ones and gets them with `# mirror: plain`. Cases that name the files
//! in their text get real ones without asking.
//!
//! The tests of nvm's own suite are cases too, see [`crate::suite`].

use crate::golden;
use crate::guard::Guard;
use crate::harness::{self, Impl, Outcome, Shell, SuiteSide, World, diff, differences};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

pub struct Case {
    pub id: String,
    /// The case file, for a case that has one.
    pub file: Option<PathBuf>,
    pub shells: Vec<Shell>,
    pub source: String,
    pub setup: String,
    pub covers: Vec<String>,
    pub body: String,
    /// Whether installs come from the mirror's real-file releases.
    pub plain_files: bool,
    /// Set for a test from nvm's own suite, which runs instead of `body`.
    pub suite: Option<crate::suite::SuiteTest>,
    /// Changes whenever the case does, see [`crate::golden`].
    pub fingerprint: String,
}

pub fn cases_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("cases")
}

/// All cases, or with `NVMIMIC_CASES=VER,DSP-02` only those whose id starts
/// with one of the listed prefixes.
pub fn load_all() -> Vec<Case> {
    let mut files: Vec<PathBuf> = fs::read_dir(cases_dir())
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sh"))
        .collect();
    files.sort();

    let filter = std::env::var("NVMIMIC_CASES").unwrap_or_default();
    let prefixes: Vec<&str> = filter.split(',').filter(|p| !p.is_empty()).collect();
    // A file of case ids, one per line, written by `cargo xtask parity
    // --changed` to name the cases a change can affect.
    let only: Option<BTreeSet<String>> = std::env::var_os("NVMIMIC_PARITY_ONLY")
        .and_then(|file| fs::read_to_string(file).ok())
        .map(|text| text.lines().map(str::to_string).collect());
    // A network pass holds only the suite tests that need the network.
    if crate::mirror::network_pass() {
        files.clear();
    }
    files
        .iter()
        .map(|file| parse(file))
        .chain(crate::suite::cases())
        .filter(|case| prefixes.is_empty() || prefixes.iter().any(|p| case.id.starts_with(p)))
        .filter(|case| only.as_ref().is_none_or(|only| only.contains(&case.id)))
        .collect()
}

fn parse(file: &Path) -> Case {
    let text = fs::read_to_string(file).unwrap();
    let mut id = None;
    let mut shells = Vec::new();
    let mut source = String::new();
    let mut setup = Vec::new();
    let mut covers = Vec::new();
    let mut body = Vec::new();
    let mut plain_files = ["npm-cli", "bin/node", "bin/npm", "node_modules/npm"]
        .iter()
        .any(|name| text.contains(name));

    for line in text.lines() {
        let header = line
            .strip_prefix("# ")
            .and_then(|rest| rest.split_once(": "))
            .filter(|_| body.is_empty());
        match header {
            Some(("id", value)) => id = Some(value.trim().to_string()),
            Some(("shells", value)) => {
                let named: Vec<&str> = value.split_whitespace().collect();
                // A case that leaves dash out does not hold there, and on
                // Debian and Ubuntu `sh` is dash.
                shells = named
                    .iter()
                    .filter_map(|name| harness::shell(name))
                    .filter(|shell| shell.kind() != "dash" || named.contains(&"dash"))
                    .collect()
            }
            Some(("source", value)) => source = value.trim().to_string(),
            Some(("setup", value)) => setup.push(value.to_string()),
            Some(("covers", value)) => covers.extend(value.split_whitespace().map(str::to_string)),
            Some(("mirror", value)) => plain_files |= value.trim() == "plain",
            _ if body.is_empty() && line.starts_with('#') => {}
            _ => body.push(line),
        }
    }

    if shells.is_empty() {
        shells = harness::shells();
    }
    Case {
        file: Some(file.to_path_buf()),
        id: id.unwrap_or_else(|| panic!("{} has no `# id:` header", file.display())),
        shells,
        source,
        setup: setup.join("\n"),
        covers,
        body: body.join("\n"),
        plain_files,
        suite: None,
        fingerprint: fingerprint(&text),
    }
}

/// A hash of a case file that changes when the case does. The `# covers:`
/// headers are left out, because they describe the case and change nothing
/// about how it runs.
fn fingerprint(text: &str) -> String {
    let mut hasher = Sha256::new();
    for line in text.lines().filter(|line| !line.starts_with("# covers: ")) {
        hasher.update(line);
        hasher.update(b"\n");
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// `<id> <shell>` pairs listed in `cases/<name>`, where the shell is a name
/// or `*`. Text after `#` is a comment.
pub fn listed(name: &str) -> BTreeSet<(String, String)> {
    listed_in(&cases_dir(), name)
}

/// [`listed`] from a folder of the caller's choosing.
pub fn listed_in(dir: &Path, name: &str) -> BTreeSet<(String, String)> {
    fs::read_to_string(dir.join(name))
        .unwrap_or_default()
        .lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (id, shell) = line.split_once(' ').unwrap_or((line, "*"));
            (id.to_string(), shell.trim().to_string())
        })
        .collect()
}

pub fn is_listed(list: &BTreeSet<(String, String)>, id: &str, shell: &str) -> bool {
    list.contains(&(id.to_string(), shell.to_string()))
        || list.contains(&(id.to_string(), "*".to_string()))
}

pub struct Result {
    pub id: String,
    pub shell: String,
    pub differences: Vec<String>,
    /// How long each side took, left then right.
    pub elapsed: (Duration, Duration),
    /// Each side's sourcing and sourcing-plus-setup times, when measured.
    pub phases: (harness::Phases, harness::Phases),
}

/// How many cases run at once: four, or fewer on a small machine.
/// `NVMIMIC_PARITY_JOBS` overrides it.
fn workers() -> usize {
    std::env::var("NVMIMIC_PARITY_JOBS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| {
            thread::available_parallelism()
                .map_or(2, |n| n.get())
                .min(4)
        })
}

/// What a run does about recordings of real nvm, see [`crate::golden`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recordings {
    /// Runs the left side every time and keeps nothing.
    Unused,
    /// Uses a recording where a good one exists, and makes one where not.
    Replay,
    /// Runs the left side every time, checks each recording against it, and
    /// writes it again.
    Refresh,
}

/// What a run compares and how much of it.
#[derive(Clone, Copy, Debug)]
pub struct Plan {
    pub left: Impl,
    pub right: Impl,
    pub recordings: Recordings,
    /// Run a case under one shell when real nvm did the same in all of them.
    pub one_shell: bool,
}

impl Plan {
    /// Both sides run for every case under every shell. This is the run for
    /// releases and for moving the nvm pin.
    pub fn full(left: Impl, right: Impl) -> Plan {
        // Official nvm against a program is what recordings are for.
        let recordings = match left == Impl::Nvm && right != Impl::Nvm {
            true => Recordings::Refresh,
            false => Recordings::Unused,
        };
        Plan {
            left,
            right,
            recordings,
            one_shell: false,
        }
    }

    /// Both sides run fresh and nothing is recorded: what nodejs.org serves
    /// is not fixed the way the test mirror's files are.
    pub fn network() -> Plan {
        Plan {
            left: Impl::Nvm,
            right: harness::NVMIMIC,
            recordings: Recordings::Unused,
            one_shell: false,
        }
    }

    /// nvmimic against recordings of real nvm.
    pub fn everyday(one_shell: bool) -> Plan {
        Plan::everyday_for(harness::NVMIMIC, one_shell)
    }

    /// Another program against the same recordings.
    pub fn everyday_for(subject: Impl, one_shell: bool) -> Plan {
        Plan {
            left: Impl::Nvm,
            right: subject,
            recordings: Recordings::Replay,
            one_shell,
        }
    }
}

/// Runs every case under its shells, the plan's left side against its right,
/// on up to four worker threads. A [`Guard`] watches macOS's Gatekeeper the
/// whole time. It can pause the workers, and when it stops the run this
/// returns the reason instead of results.
pub fn run_all(cases: &[Case], plan: Plan) -> std::result::Result<Vec<Result>, String> {
    let Plan { left, right, .. } = plan;
    let results = Mutex::new(Vec::new());
    let next = Mutex::new(0usize);
    let workers = workers();

    let labels = if left == right {
        (
            format!("{} run 1", left.name()),
            format!("{} run 2", right.name()),
        )
    } else {
        (left.name().to_string(), right.name().to_string())
    };
    let labels = &labels;

    // Building the mirror first keeps its one-time work out of the guarded part.
    crate::mirror::shared();
    let rotation = plan.one_shell.then(golden::rotation);
    let guard = Guard::start(&crate::guard::stage_dir());
    let guard = &guard;
    let suite_root = World::new().root;
    let suite_root = &suite_root;
    let (replayed, recorded, narrowed, sandboxes) = (
        AtomicUsize::new(0),
        AtomicUsize::new(0),
        AtomicUsize::new(0),
        AtomicUsize::new(0),
    );

    thread::scope(|scope| {
        for worker in 0..workers {
            let (results, next) = (&results, &next);
            let (replayed, recorded, narrowed, sandboxes) =
                (&replayed, &recorded, &narrowed, &sandboxes);
            scope.spawn(move || {
                // One reusable sandbox per side for nvm's own suite.
                let mut suite_left = SuiteSide::new(suite_root, &format!("suite-{worker}-left"));
                let mut suite_right = SuiteSide::new(suite_root, &format!("suite-{worker}-right"));
                loop {
                    let index = {
                        let mut next = next.lock().unwrap();
                        let index = *next;
                        *next += 1;
                        index
                    };
                    let Some(case) = cases.get(index) else {
                        break;
                    };
                    let world = World::new();

                    let mut stored: Vec<Option<Outcome>> = case
                        .shells
                        .iter()
                        .map(|shell| match plan.recordings {
                            Recordings::Unused => None,
                            _ => golden::load(case, shell),
                        })
                        .collect();
                    let only = rotation
                        .filter(|_| {
                            let all: Option<Vec<(&Shell, &Outcome)>> = case
                                .shells
                                .iter()
                                .zip(&stored)
                                .map(|(shell, outcome)| Some((shell, outcome.as_ref()?)))
                                .collect();
                            let differing = all.map(|all| golden::shell_differences(case, &all));
                            if std::env::var_os("NVMIMIC_PARITY_EXPLAIN").is_some()
                                && let Some(differing) = &differing
                                && !differing.is_empty()
                            {
                                let names: Vec<&str> =
                                    differing.iter().map(String::as_str).collect();
                                eprintln!("{} differs by shell in: {}", case.id, names.join(" "));
                            }
                            case.shells.len() > 1 && differing.is_some_and(|d| d.is_empty())
                        })
                        .map(|rotation| golden::pick(case, rotation));
                    if only.is_some() {
                        narrowed.fetch_add(1, Ordering::SeqCst);
                    }

                    for (index, shell) in case.shells.iter().enumerate() {
                        if only.is_some_and(|only| only != index) {
                            continue;
                        }
                        if guard.wait_turn().is_err() {
                            return;
                        }
                        let mut stale = None;
                        let a = match (plan.recordings, stored[index].take()) {
                            (Recordings::Replay, Some(outcome)) => {
                                replayed.fetch_add(1, Ordering::SeqCst);
                                outcome
                            }
                            (recordings, old) => {
                                let fresh = world.run_case(left, shell, case, &mut suite_left);
                                if recordings != Recordings::Unused {
                                    stale = old
                                        .map(|old| diff(("recorded", &old), ("now", &fresh)))
                                        .filter(|changes| !changes.is_empty());
                                    golden::store(case, shell, &fresh);
                                    recorded.fetch_add(1, Ordering::SeqCst);
                                }
                                fresh
                            }
                        };
                        let b = world.run_case(right, shell, case, &mut suite_right);
                        // What the right side differs in on purpose is
                        // left out, rule by rule.
                        let expected = match right {
                            Impl::Subject(subject) => Some(subject.expected()),
                            Impl::Nvm => None,
                        };
                        let mut differences: Vec<String> =
                            differences((&labels.0, &a), (&labels.1, &b))
                                .into_iter()
                                .filter(|(key, _)| {
                                    !expected.is_some_and(|expected| {
                                        expected.covers(&case.id, &shell.name, key)
                                    })
                                })
                                .map(|(_, text)| text)
                                .collect();
                        if let Some(changes) = stale {
                            differences.push(format!(
                                "real nvm no longer does what was recorded for this pair, so \
                                 everyday runs compared against a stale recording:\n{}",
                                changes.join("\n")
                            ));
                        }
                        results.lock().unwrap().push(Result {
                            id: case.id.clone(),
                            shell: shell.name.clone(),
                            differences,
                            elapsed: (a.elapsed, b.elapsed),
                            phases: (a.phases, b.phases),
                        });
                    }
                }
                let used = suite_left.used() + suite_right.used();
                sandboxes.fetch_add(used as usize, Ordering::SeqCst);
            });
        }
    });

    eprintln!("{}", guard.finish());
    if plan.recordings != Recordings::Unused {
        eprintln!(
            "recordings of real nvm: {} replayed, {} recorded",
            replayed.into_inner(),
            recorded.into_inner()
        );
    }
    if let Some(rotation) = rotation {
        eprintln!(
            "{} of {} cases ran under one shell (rotation {rotation})",
            narrowed.into_inner(),
            cases.len()
        );
    }
    if cases.iter().any(|case| case.suite.is_some()) {
        eprintln!("suite sandboxes used: {}", sandboxes.into_inner());
    }
    if let Some(reason) = guard.stopped() {
        return Err(reason);
    }
    crate::mirror::shared().verify()?;
    let mut results = results.into_inner().unwrap();
    results.sort_by(|a, b| (&a.id, &a.shell).cmp(&(&b.id, &b.shell)));
    Ok(results)
}

/// Measures what each case's body runs in nvm.sh: every case once, under
/// its first shell, against a traced copy of real nvm. Goes through the same
/// guard as a parity run.
pub fn measure_covers(
    cases: &[Case],
) -> std::result::Result<BTreeMap<String, BTreeSet<String>>, String> {
    let covers = Mutex::new(BTreeMap::new());
    let next = AtomicUsize::new(0);
    crate::mirror::shared();
    let guard = Guard::start(&crate::guard::stage_dir());
    let guard = &guard;
    let suite_root = World::new().root;
    let suite_root = &suite_root;

    thread::scope(|scope| {
        for worker in 0..workers() {
            let (covers, next) = (&covers, &next);
            scope.spawn(move || {
                let mut suite_side = SuiteSide::new(suite_root, &format!("suite-{worker}"));
                while let Some(case) = cases.get(next.fetch_add(1, Ordering::SeqCst)) {
                    if guard.wait_turn().is_err() {
                        return;
                    }
                    let mut world = World::new();
                    world.options.trace_covers = true;
                    let outcome = world.run_case(Impl::Nvm, &case.shells[0], case, &mut suite_side);
                    covers
                        .lock()
                        .unwrap()
                        .insert(case.id.clone(), outcome.covered);
                }
            });
        }
    });

    eprintln!("{}", guard.finish());
    if let Some(reason) = guard.stopped() {
        return Err(reason);
    }
    Ok(covers.into_inner().unwrap())
}

/// Writes `<id> <shell>` lines for every result that differs, under a header.
pub fn record(name: &str, header: &str, results: &[Result]) {
    record_in(&cases_dir(), name, header, results)
}

/// [`record`] into a folder of the caller's choosing, for a program that
/// keeps its own lists.
pub fn record_in(dir: &Path, name: &str, header: &str, results: &[Result]) {
    let mut text = String::from(header);
    for result in results.iter().filter(|r| !r.differences.is_empty()) {
        text.push_str(&format!("{} {}\n", result.id, result.shell));
    }
    fs::write(dir.join(name), text).unwrap();
}

/// The rule a run passes by: every pair matches unless it is on the `known`
/// list, and a listed pair that now matches fails too, so the list only
/// ever shrinks. `list` is the list's name, for the message.
pub fn judge(
    results: &[Result],
    known: &BTreeSet<(String, String)>,
    list: &str,
) -> std::result::Result<(), String> {
    let mut unexpected = Vec::new();
    let mut fixed = Vec::new();
    for result in results {
        let listed = is_listed(known, &result.id, &result.shell);
        match (result.differences.is_empty(), listed) {
            (false, false) => unexpected.push(format!(
                "{} ({}):\n{}",
                result.id,
                result.shell,
                result.differences.join("\n")
            )),
            (true, true) => fixed.push(format!("{} {}", result.id, result.shell)),
            _ => {}
        }
    }
    if unexpected.is_empty() && fixed.is_empty() {
        return Ok(());
    }
    Err(format!(
        "unexpected differences:\n{}\n\nnow passing, remove from {list}:\n{}",
        unexpected.join("\n\n"),
        fixed.join("\n")
    ))
}

/// Writes every difference of a run to `path`, one block per pair.
pub fn write_report(results: &[Result], path: &Path) {
    let mut text = String::new();
    for result in results.iter().filter(|r| !r.differences.is_empty()) {
        text.push_str(&format!(
            "==== {} ({})\n{}\n\n",
            result.id,
            result.shell,
            result.differences.join("\n")
        ));
    }
    let _ = fs::write(path, text);
}

/// Writes how long every pair took to `path`: milliseconds per side, then
/// each side's time to source nvm.sh and to finish setup when
/// `NVMIMIC_PARITY_PHASES` is set.
pub fn write_timings(results: &[Result], path: &Path) {
    let ms = |d: Duration| d.as_millis().to_string();
    let phase = |p: harness::Phases| match p {
        Some((sourced, ready)) => format!("{}\t{}", ms(sourced), ms(ready)),
        None => "\t".to_string(),
    };
    let mut text = String::from(
        "id\tshell\tleft_ms\tright_ms\tleft_sourced\tleft_ready\tright_sourced\tright_ready\n",
    );
    for r in results {
        text.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            r.id,
            r.shell,
            ms(r.elapsed.0),
            ms(r.elapsed.1),
            phase(r.phases.0),
            phase(r.phases.1)
        ));
    }
    let _ = fs::write(path, text);
}
