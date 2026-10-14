//! Parity tests. The full runs take minutes, so they are ignored by default
//! and run through `cargo xtask parity`.

use nvmimic_parity::cases::{self, Plan, Result};
use nvmimic_parity::harness::{self, Impl, World};

const KNOWN_FAILING: &str = "KNOWN_FAILING";
const NONDETERMINISTIC: &str = "NONDETERMINISTIC";

fn recording() -> bool {
    std::env::var_os("NVMIMIC_PARITY_RECORD").is_some()
}

/// `NVMIMIC_PARITY_RUN` picks how much runs: `everyday` replays recordings of
/// official nvm and runs shell-independent cases under one shell, `all-shells`
/// replays and runs every shell, and anything else is the full run.
fn plan() -> Plan {
    if nvmimic_parity::mirror::network_pass() {
        return Plan::network();
    }
    match std::env::var("NVMIMIC_PARITY_RUN").as_deref() {
        Ok("everyday") if !recording() => Plan::everyday(true),
        Ok("all-shells") if !recording() => Plan::everyday(false),
        _ => Plan::full(Impl::Nvm, harness::NVMIMIC),
    }
}

/// The Gatekeeper guard stopped the run before it finished.
fn stopped(reason: String) -> Vec<Result> {
    panic!("the run was stopped to protect the machine: {reason}");
}

fn report(results: &[Result]) -> usize {
    let passing = results.iter().filter(|r| r.differences.is_empty()).count();
    eprintln!("{passing}/{} case-shell pairs match", results.len());
    // A subject with rules of its own: say how many pairs match only
    // because a rule left a difference out. nvmimic has no rules.
    let under_rules = results
        .iter()
        .filter(|r| r.differences.is_empty() && r.covered > 0)
        .count();
    if under_rules > 0 {
        eprintln!("{under_rules} of them match only with differences a rule leaves out");
    }
    passing
}

/// The harness itself works: official nvm sources and answers in every shell.
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

/// Official nvm run twice must agree with itself. Pairs that do not are
/// nondeterministic and get listed in `cases/NONDETERMINISTIC`, which the
/// comparison against nvmimic skips. `NVMIMIC_PARITY_RECORD=1` rewrites the list.
#[test]
#[ignore = "full run, use `cargo xtask parity determinism`"]
fn nvm_is_deterministic() {
    let results = cases::run_all(&cases::load_all(), Plan::full(Impl::Nvm, Impl::Nvm))
        .unwrap_or_else(stopped);
    report(&results);
    if recording() {
        cases::record(
            NONDETERMINISTIC,
            "# Case-shell pairs where official nvm disagrees with itself across two runs.\n\
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
    let mut results = cases::run_all(&all, plan()).unwrap_or_else(stopped);
    results.retain(|r| !cases::is_listed(&skipped, &r.id, &r.shell));
    report(&results);
    let target = harness::target_dir();
    cases::write_report(&results, &target.join("parity-report.txt"));
    cases::write_timings(&results, &target.join("parity-timings.tsv"));

    if recording() {
        cases::record(
            KNOWN_FAILING,
            "# Case-shell pairs where nvmimic does not match official nvm yet.\n\
             # Regenerate with `cargo xtask parity --record`. The list only shrinks.\n",
            &results,
        );
        return;
    }

    let known = cases::listed(KNOWN_FAILING);
    if let Err(message) = cases::judge(&results, &known, KNOWN_FAILING) {
        panic!("{message}");
    }
}

/// Measures which parts of nvm.sh every case runs and writes the result
/// into the `# covers:` headers and `cases/NVM_SUITE_COVERS`.
#[test]
#[ignore = "rewrites case files, use `cargo xtask parity covers`"]
fn record_covers() {
    let all = cases::load_all();
    let measured = cases::measure_covers(&all).unwrap_or_else(|reason| {
        panic!("the run was stopped to protect the machine: {reason}");
    });
    let mut rewritten = 0;
    let mut suite = nvmimic_parity::covers::suite_covers()
        .into_iter()
        .map(|(id, names)| (id, names.into_iter().collect()))
        .collect::<std::collections::BTreeMap<_, std::collections::BTreeSet<String>>>();
    for case in &all {
        let names = &measured[&case.id];
        match &case.file {
            Some(file) => rewritten += nvmimic_parity::covers::write_header(file, names) as usize,
            None => {
                suite.insert(case.id.clone(), names.clone());
            }
        }
    }
    nvmimic_parity::covers::write_suite_covers(&suite);
    eprintln!(
        "covers measured for {} cases, {rewritten} case files rewritten",
        all.len()
    );
}

/// The release check: nothing is left in `cases/KNOWN_FAILING`.
#[test]
#[ignore = "fails until every case matches"]
fn no_known_failures() {
    let known = cases::listed(KNOWN_FAILING);
    assert!(known.is_empty(), "{} known failures remain", known.len());
}

/// Shell text that puts a compiled `node` in place of the script each
/// stand-in release comes with, so its answer can be remembered. One shared
/// file, linked into every sandbox under the name `node`.
fn compiled_node(versions: &[&str]) -> String {
    let shared = harness::repo_root().join("target/parity-fake/node");
    std::fs::create_dir_all(shared.parent().unwrap()).unwrap();
    let _ = std::fs::remove_file(&shared);
    std::fs::hard_link(env!("CARGO_BIN_EXE_fake_node"), &shared).unwrap();
    versions
        .iter()
        .map(|version| {
            format!(
                "rm -f \"$NVM_DIR/versions/node/v{version}/bin/node\"\nln -s '{}' \"$NVM_DIR/versions/node/v{version}/bin/node\"\n",
                shared.display()
            )
        })
        .collect()
}

/// Everyday commands start nvmimic once each and nothing else: no node, no
/// sed or awk, and node and npm only when their versions are not
/// remembered yet.
#[test]
#[ignore = "full run, use `cargo xtask parity launches`"]
fn everyday_commands_launch_only_nvmimic() {
    let mut world = World::new();
    world.options.count_launches = true;
    let compiled = compiled_node(&["24.15.0", "20.20.2"]);
    // Both versions get used once, so every answer is remembered before
    // the counted step.
    let installed = format!(
        "nvm install 24.15.0\nnvm install 20.20.2\n{compiled}nvm use 24.15.0\nnvm use 20.20.2"
    );
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
        // bash sets SHELL for itself when nothing handed it one, and a
        // variable that is set and not exported costs a second start.
        (
            "export SHELL=\"${SHELL-/bin/sh}\"\n. \"$NVM_DIR/nvm.sh\"",
            1,
        ),
        (". \"$NVM_DIR/nvm.sh\" --no-use", 1),
    ];
    for shell in harness::shells() {
        for (step, expected) in steps {
            let outcome = world.run_with(harness::NVMIMIC, &shell, "", &installed, step);
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

        // With nothing remembered, the first `nvm use` asks node and npm for
        // their versions and the second one reuses the answers.
        let outcome = world.run_with(
            harness::NVMIMIC,
            &shell,
            "",
            &format!("nvm install 24.15.0\n{compiled}rm -rf \"$HOME/../own/cache\""),
            "nvm use 24.15.0\nnvm use 24.15.0",
        );
        let npm = outcome.launches.iter().filter(|l| l.ends_with("/bin/npm"));
        let node = outcome.launches.iter().filter(|l| l.ends_with("/bin/node"));
        let ours = outcome.launches.iter().filter(|l| *l == "nvmimic");
        assert!(
            npm.count() == 1 && node.count() == 1 && ours.count() == 2,
            "{}: node and npm should each run once for two `nvm use` calls, launched {:?}",
            shell.name,
            outcome.launches
        );

        // A `node` that is a script is never remembered: it could be a
        // version manager's stand-in that answers differently next time.
        // The stand-in releases come with such a script.
        let outcome = world.run_with(
            harness::NVMIMIC,
            &shell,
            "",
            "nvm install 24.15.0",
            "nvm current\nnvm current",
        );
        let node = outcome.launches.iter().filter(|l| l.ends_with("/bin/node"));
        assert!(
            node.count() == 2,
            "{}: a script named node should be asked on every call, launched {:?}",
            shell.name,
            outcome.launches
        );

        // One TAB starts nvmimic once and nothing else.
        if shell.name == "bash" {
            let outcome = world.run_with(
                harness::NVMIMIC,
                &shell,
                "",
                "nvm install 24.15.0",
                ". \"$NVM_DIR/bash_completion\"\nCOMP_WORDS=(nvm use \"\")\nCOMP_CWORD=2\n__nvm",
            );
            assert_eq!(
                outcome.launches,
                ["nvmimic"],
                "{}: one TAB should start nvmimic once",
                shell.name
            );
        }
    }
}
