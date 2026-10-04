//! nvm's own test suite as parity cases. Every executable file under the
//! vendored `test/` directory is one test. Each side keeps one copy of that
//! directory at `$NVM_DIR/test`, where the tests expect it, and runs every
//! test there the way nvm's test runner (urchin 0.0.5) does. A test that
//! passes against real nvm has to behave the same against nvmimic.
//!
//! Tests that cannot be compared are listed in `cases/NVM_SUITE_SKIPPED`,
//! each with the reason.

use crate::cases::Case;
use crate::harness;
use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// The file listing skipped tests: `<reason>: <path under test/>` per line,
/// where a path ending in `/` skips a whole directory.
pub const SKIPPED: &str = "NVM_SUITE_SKIPPED";

/// Files urchin runs around tests rather than as tests.
const HOOKS: &[&str] = &["setup", "setup_dir", "teardown", "teardown_dir"];

/// The script that runs one test.
///
/// It follows urchin 0.0.5: on the way down to the test, each directory's
/// `setup_dir` and `setup` run, `setup` runs once more beside the test
/// itself, and the matching `teardown` and `teardown_dir` run on the way
/// back. What the hooks print is dropped.
///
/// One thing differs from urchin on purpose. urchin starts hooks, and tests
/// with their own `#!` line, as programs. Here every script is handed to its
/// interpreter as a file to read instead: `sh ./setup` rather than `./setup`.
/// The script behaves the same, and macOS does not treat it as a new program
/// that has to be scanned before it may start.
pub const RUNNER: &str = r#"# usage: sh run-nvm-test <shell> <directory>... <test>
shell="$1"
shift

hook() {
  if [ -f "$1" ] && [ -x "$1" ]; then
    /bin/sh "./$1" >/dev/null 2>&1
  fi
  return 0
}

# Runs a test under the shell being tested when its first line asks for sh
# or for nothing, and under the interpreter that line names otherwise.
run_test() {
  IFS= read -r first < "$1" || first=''
  case "$first" in
    '#!/bin/sh'* | '#!/usr/bin/env sh'*) TEST_SHELL=/bin/sh "$shell" "$1" ;;
    '#!'*)
      # shellcheck disable=SC2086
      TEST_SHELL=/bin/sh ${first#??} "./$1"
      ;;
    *) TEST_SHELL=/bin/sh "$shell" "$1" ;;
  esac
}

descend() {
  if [ "$#" -eq 1 ]; then
    hook setup
    run_test "$1"
    rc=$?
    hook teardown
    return "$rc"
  fi
  (
    cd -- "$1" || exit 97
    shift
    hook setup_dir
    hook setup
    descend "$@"
    rc=$?
    hook teardown
    hook teardown_dir
    exit "$rc"
  )
}

cd "$NVM_DIR/test" || exit 97
descend "$@"
"#;

pub fn tree() -> PathBuf {
    crate::reference_dir().join("test")
}

/// One suite test as a parity case.
pub struct SuiteTest {
    /// Path under `test/`, such as `fast/Unit tests/nvm_alias_path`.
    pub path: String,
    /// Whether the script turns on shell tracing. Traces show the inside of
    /// `nvm.sh`, which differs by design, and tests send them to stdout as
    /// well as stderr, so only the exit status and files are compared then.
    pub traces: bool,
}

impl SuiteTest {
    /// The shell line that runs this test, for a shell at `shell`.
    pub fn command(&self, shell: &Path) -> String {
        let mut line = format!(
            "/bin/sh {} {}",
            quote(&harness::suite_runner().display().to_string()),
            quote(&shell.display().to_string())
        );
        for part in self.path.split('/') {
            line.push(' ');
            line.push_str(&quote(part));
        }
        if self.traces {
            line.push_str(" >/dev/null 2>&1");
        }
        line
    }
}

fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

/// A case id made from a test's path: `NVT-fast/Unit_tests/nvm_alias_path`.
fn id(path: &str) -> String {
    let safe: String = path
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '/' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("NVT-{safe}")
}

/// The skip list: `(reason, path)` pairs.
pub fn skipped() -> Vec<(String, String)> {
    fs::read_to_string(crate::cases::cases_dir().join(SKIPPED))
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .filter_map(|line| line.split_once(": "))
        .map(|(reason, path)| (reason.trim().to_string(), path.to_string()))
        .collect()
}

/// The reason a test is on the skip list, if it is.
fn skip_reason<'a>(skipped: &'a [(String, String)], path: &str) -> Option<&'a str> {
    // A line naming the test itself wins over one naming its directory.
    skipped
        .iter()
        .find(|(_, entry)| entry == path)
        .or_else(|| {
            skipped
                .iter()
                .find(|(_, entry)| entry.ends_with('/') && path.starts_with(entry.as_str()))
        })
        .map(|(reason, _)| reason.as_str())
}

/// The reason given to tests that install real Node releases. A network
/// pass runs exactly these, and every other run leaves them out.
const REAL_NODE: &str = "real-node";

/// Whether a test belongs in this run.
fn runs_now(skipped: &[(String, String)], path: &str) -> bool {
    match (crate::mirror::network_pass(), skip_reason(skipped, path)) {
        (true, reason) => reason == Some(REAL_NODE),
        (false, reason) => reason.is_none(),
    }
}

/// Every test in the vendored suite, skipped ones included, sorted by path.
pub fn all_tests() -> Vec<String> {
    let root = tree();
    let mut found = Vec::new();
    let mut pending = vec![root.clone()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = fs::metadata(&path) else {
                continue;
            };
            let name = entry.file_name().to_string_lossy().into_owned();
            if meta.is_dir() {
                pending.push(path);
            } else if meta.permissions().mode() & 0o111 != 0 && !HOOKS.contains(&name.as_str()) {
                found.push(path.strip_prefix(&root).unwrap().display().to_string());
            }
        }
    }
    found.sort();
    found
}

/// The suite's tests as cases, without the skipped ones.
pub fn cases() -> Vec<Case> {
    let skipped = skipped();
    let names = crate::nvm_function_names();
    let measured = crate::covers::suite_covers();
    let mut ids = BTreeSet::new();
    all_tests()
        .into_iter()
        .filter(|path| runs_now(&skipped, path))
        .map(|path| {
            let text = fs::read_to_string(tree().join(&path)).unwrap_or_default();
            let traces = text.lines().any(|line| {
                let line = line.trim_start();
                line.strip_prefix("set -").is_some_and(|flags| {
                    flags.split_whitespace().next().unwrap_or("").contains('x')
                })
            });
            // Tests with their own interpreter behave the same under every
            // shell, so one run is enough.
            let own_interpreter = text.starts_with("#!")
                && !text.starts_with("#!/bin/sh")
                && !text.starts_with("#!/usr/bin/env sh");
            let shells = if own_interpreter {
                harness::shells().into_iter().take(1).collect()
            } else {
                harness::shells()
            };
            let file_name = path.rsplit('/').next().unwrap_or("");
            let id = id(&path);
            // Until a test's covers are measured, its file name often names
            // the one function it is about.
            let covers = measured.get(&id).cloned().unwrap_or_else(|| {
                file_name
                    .split(' ')
                    .next()
                    .filter(|word| names.iter().any(|name| name == word))
                    .map(|word| vec![word.to_string()])
                    .unwrap_or_default()
            });
            assert!(ids.insert(id.clone()), "two suite tests share the id {id}");
            Case {
                // The test's own text is part of the vendored nvm, which
                // every recording's key covers already.
                fingerprint: path.clone(),
                file: None,
                id,
                shells,
                source: String::new(),
                setup: String::new(),
                covers,
                body: String::new(),
                plain_files: false,
                suite: Some(SuiteTest { path, traces }),
            }
        })
        .collect()
}
