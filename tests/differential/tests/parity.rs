//! Parity tests. The full runs take minutes, so they are ignored by default
//! and run through `cargo xtask parity`.

use nvmimic_parity::cases::{self, Result};
use nvmimic_parity::harness::{self, Impl, World};

const KNOWN_FAILING: &str = "KNOWN_FAILING";
const NONDETERMINISTIC: &str = "NONDETERMINISTIC";

fn recording() -> bool {
    std::env::var_os("NVMIMIC_PARITY_RECORD").is_some()
}

/// The Gatekeeper guard stopped the run before it finished.
fn stopped(reason: String) -> Vec<Result> {
    panic!("the run was stopped to protect the machine: {reason}");
}

fn report(results: &[Result]) -> usize {
    let passing = results.iter().filter(|r| r.differences.is_empty()).count();
    eprintln!("{passing}/{} case-shell pairs match", results.len());
    passing
}

/// Every difference from the last run, in `target/parity-report.txt`.
fn write_report(results: &[Result]) {
    let mut text = String::new();
    for result in results.iter().filter(|r| !r.differences.is_empty()) {
        text.push_str(&format!(
            "==== {} ({})\n{}\n\n",
            result.id,
            result.shell,
            result.differences.join("\n")
        ));
    }
    let path = harness::repo_root().join("target/parity-report.txt");
    let _ = std::fs::write(path, text);
}

/// The harness itself works: real nvm sources and answers in every shell.
#[test]
fn harness_runs_real_nvm() {
    let world = World::new();
    for shell in harness::shells() {
        let outcome = world.run(Impl::Nvm, &shell, "nvm --version");
        assert_eq!(outcome.code, Some(0), "{}: {}", shell.name, outcome.stderr);
        assert_eq!(outcome.stdout, format!("{}\n", nvmimic::pin::VERSION));
        let defined = outcome.state["defined"].split(' ').count();
        assert_eq!(
            defined,
            nvmimic_parity::nvm_function_names().len(),
            "{}",
            shell.name
        );
        assert_eq!(
            outcome.state["var:NVM_DIR"].trim_matches('\''),
            "<home>/.nvm"
        );
        assert!(outcome.state.contains_key("fs:home/.nvm"), "{}", shell.name);
    }
}

/// The probe sees variables a command leaves behind, in every shell.
#[test]
fn probe_sees_leaked_variables() {
    let world = World::new();
    for shell in harness::shells() {
        let outcome = world.run(Impl::Nvm, &shell, "nvm --help >/dev/null");
        assert_eq!(
            outcome
                .state
                .get("var:NVM_VERSION")
                .map(|v| v.trim_matches('\'')),
            Some(nvmimic::pin::VERSION),
            "{}",
            shell.name
        );
    }
}

/// Real nvm run twice must agree with itself. Pairs that do not are
/// nondeterministic and get listed in `cases/NONDETERMINISTIC`, which the
/// comparison against nvmimic skips. `NVMIMIC_PARITY_RECORD=1` rewrites the list.
#[test]
#[ignore = "full run, use `cargo xtask parity determinism`"]
fn nvm_is_deterministic() {
    let results = cases::run_all(&cases::load_all(), Impl::Nvm, Impl::Nvm).unwrap_or_else(stopped);
    report(&results);
    if recording() {
        cases::record(
            NONDETERMINISTIC,
            "# Case-shell pairs where real nvm disagrees with itself across two runs.\n\
             # Regenerate with `cargo xtask parity determinism --record`.\n",
            &results,
        );
        return;
    }
    let listed = cases::listed(NONDETERMINISTIC);
    let unexpected: Vec<String> = results
        .iter()
        .filter(|r| !r.differences.is_empty())
        .filter(|r| !cases::is_listed(&listed, &r.id, &r.shell))
        .map(|r| format!("{} ({}):\n{}", r.id, r.shell, r.differences.join("\n")))
        .collect();
    assert!(
        unexpected.is_empty(),
        "nvm disagrees with itself:\n{}",
        unexpected.join("\n\n")
    );
}

/// Every reference case must match, except pairs listed in
/// `cases/KNOWN_FAILING`. A listed pair that now matches also fails, so the
/// list only ever shrinks. `NVMIMIC_PARITY_RECORD=1` rewrites the list.
#[test]
#[ignore = "full run, use `cargo xtask parity`"]
fn reference_cases_match_or_are_known() {
    let skipped = cases::listed(NONDETERMINISTIC);
    let all = cases::load_all();
    let mut results = cases::run_all(&all, Impl::Nvm, Impl::Nvmimic).unwrap_or_else(stopped);
    results.retain(|r| !cases::is_listed(&skipped, &r.id, &r.shell));
    report(&results);
    write_report(&results);

    if recording() {
        cases::record(
            KNOWN_FAILING,
            "# Case-shell pairs where nvmimic does not match real nvm yet.\n\
             # Regenerate with `cargo xtask parity --record`. The list only shrinks.\n",
            &results,
        );
        return;
    }

    let known = cases::listed(KNOWN_FAILING);
    let mut unexpected = Vec::new();
    let mut fixed = Vec::new();
    for result in &results {
        let listed = cases::is_listed(&known, &result.id, &result.shell);
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
    assert!(
        unexpected.is_empty() && fixed.is_empty(),
        "unexpected differences:\n{}\n\nnow passing, remove from KNOWN_FAILING:\n{}",
        unexpected.join("\n\n"),
        fixed.join("\n")
    );
}

/// The release check: nothing is left in `cases/KNOWN_FAILING`.
#[test]
#[ignore = "fails until every case matches"]
fn no_known_failures() {
    let known = cases::listed(KNOWN_FAILING);
    assert!(known.is_empty(), "{} known failures remain", known.len());
}

/// Everyday commands start nvmimic once each and nothing else: no node, no
/// npm, no sed or awk.
#[test]
#[ignore = "full run, use `cargo xtask parity launches`"]
fn everyday_commands_launch_only_nvmimic() {
    let mut world = World::new();
    world.options.count_launches = true;
    let steps: &[(&str, usize)] = &[
        ("nvm current", 1),
        ("nvm ls", 1),
        ("nvm ls 24", 1),
        ("nvm which 24.15.0", 1),
        ("nvm version 24", 1),
        ("nvm use 24.15.0", 1),
        ("nvm use system", 1),
        ("nvm deactivate", 1),
        ("nvm alias dev 24.15.0", 1),
        ("nvm alias", 1),
        ("nvm unalias dev", 1),
        ("nvm_find_nvmrc", 0),
        (". \"$NVM_DIR/nvm.sh\"", 1),
    ];
    for shell in harness::shells() {
        for (step, expected) in steps {
            let outcome = world.run_with(
                Impl::Nvmimic,
                &shell,
                "",
                "nvm install 24.15.0\nnvm install 20.20.2",
                step,
            );
            let foreign: Vec<&String> = outcome
                .launches
                .iter()
                .filter(|l| *l != "nvmimic")
                .collect();
            let ours = outcome.launches.len() - foreign.len();
            assert!(
                foreign.is_empty() && ours == *expected,
                "{} `{step}`: launched {:?}, expected nvmimic {expected} time(s)",
                shell.name,
                outcome.launches
            );
        }
    }
}
