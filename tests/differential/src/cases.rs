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

use crate::guard::Guard;
use crate::harness::{self, Impl, Shell, World, diff};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;

pub struct Case {
    pub id: String,
    pub shells: Vec<Shell>,
    pub source: String,
    pub setup: String,
    pub covers: Vec<String>,
    pub body: String,
    /// Whether installs come from the mirror's real-file releases.
    pub plain_files: bool,
    /// Set for a test from nvm's own suite, which runs instead of `body`.
    pub suite: Option<crate::suite::SuiteTest>,
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
    files
        .iter()
        .map(|file| parse(file))
        .chain(crate::suite::cases())
        .filter(|case| prefixes.is_empty() || prefixes.iter().any(|p| case.id.starts_with(p)))
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
                shells = value
                    .split_whitespace()
                    .filter_map(harness::shell)
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
        id: id.unwrap_or_else(|| panic!("{} has no `# id:` header", file.display())),
        shells,
        source,
        setup: setup.join("\n"),
        covers,
        body: body.join("\n"),
        plain_files,
        suite: None,
    }
}

/// `<id> <shell>` pairs listed in `cases/<name>`, where the shell is a name
/// or `*`. Text after `#` is a comment.
pub fn listed(name: &str) -> BTreeSet<(String, String)> {
    fs::read_to_string(cases_dir().join(name))
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
}

/// Runs every case under each of its shells, `left` against `right`, on up to
/// four worker threads. A [`Guard`] watches macOS's Gatekeeper the whole
/// time. It can pause the workers, and when it stops the run this returns
/// the reason instead of results.
pub fn run_all(
    cases: &[Case],
    left: Impl,
    right: Impl,
) -> std::result::Result<Vec<Result>, String> {
    let results = Mutex::new(Vec::new());
    let next = Mutex::new(0usize);
    let workers = std::env::var("NVMIMIC_PARITY_JOBS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| {
            thread::available_parallelism()
                .map_or(2, |n| n.get())
                .min(4)
        });

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
    let guard = Guard::start(&crate::guard::stage_dir());
    let guard = &guard;
    let suite_root = World::new().root;
    let suite_root = &suite_root;

    thread::scope(|scope| {
        for worker in 0..workers {
            let (results, next) = (&results, &next);
            scope.spawn(move || {
                // One reusable sandbox per side for nvm's own suite.
                let suite_left = suite_root.join(format!("suite-{worker}-left"));
                let suite_right = suite_root.join(format!("suite-{worker}-right"));
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
                    for shell in &case.shells {
                        if guard.wait_turn().is_err() {
                            return;
                        }
                        let a = world.run_case(left, shell, case, &suite_left);
                        let b = world.run_case(right, shell, case, &suite_right);
                        let differences = diff((&labels.0, &a), (&labels.1, &b));
                        results.lock().unwrap().push(Result {
                            id: case.id.clone(),
                            shell: shell.name.clone(),
                            differences,
                        });
                    }
                }
            });
        }
    });

    eprintln!("{}", guard.finish());
    if let Some(reason) = guard.stopped() {
        return Err(reason);
    }
    crate::mirror::shared().verify()?;
    let mut results = results.into_inner().unwrap();
    results.sort_by(|a, b| (&a.id, &a.shell).cmp(&(&b.id, &b.shell)));
    Ok(results)
}

/// Writes `<id> <shell>` lines for every result that differs, under a header.
pub fn record(name: &str, header: &str, results: &[Result]) {
    let mut text = String::from(header);
    for result in results.iter().filter(|r| !r.differences.is_empty()) {
        text.push_str(&format!("{} {}\n", result.id, result.shell));
    }
    fs::write(cases_dir().join(name), text).unwrap();
}
