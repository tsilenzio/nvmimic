//! Project tasks: `cargo xtask <task>`.

mod bench;
mod changed;
mod gen_shell;
mod nvm_diff;
mod string_audit;
mod trace;

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "\
usage: cargo xtask <task>

tasks:
  parity                            the everyday run: nvmimic against recordings of real nvm
  parity --all-shells               the same, with every case under every shell
  parity --changed                  the everyday run on the cases a change can affect
  parity --full [--record]          run real nvm and nvmimic side by side on everything
  parity --network                  run nvm's tests that install real Node, against nodejs.org
  parity covers                     measure what each case runs in nvm.sh, write `# covers:`
  parity determinism [--record]     run real nvm against itself on every case
  parity launches                   check everyday commands start only nvmimic
  smoke                             install nvmimic and three real Node releases, then use them
  bench [--check] [--runs=N]        time sourcing nvm.sh for real nvm and nvmimic
  gen-shell [--check]               write shell/nvm.sh from the pinned nvm.sh
  trace [--strict]                  map nvm.sh's functions to nvmimic's code and cases
  nvm-diff <old> <new> [--repo P]   report what changed in nvm between two revisions
  string-audit                      check nvmimic's messages against the pinned nvm.sh
  help                              show this list

The everyday run records what real nvm does the first time and reads it back
after that. It records a case again when the case, the vendored nvm, the shell
or the OS version changes. Cases where real nvm did the same in every shell
run under one shell, a different one each run. --full trusts no recording and
is the run for releases and for moving the nvm pin. --changed compares the
source files with the last run that passed and runs the cases whose `# covers:`
headers name an nvm function ported in a changed file.
--record rewrites cases/KNOWN_FAILING or cases/NONDETERMINISTIC from a full run.
--network runs only the tests of nvm's own suite that download real Node
releases. Each file is fetched from nodejs.org once and kept in
target/parity-network/. Start with one test (NVMIMIC_CASES) and read the
`gatekeeper guard:` line before running more.
NVMIMIC_CASES=VER,DSP-02 limits a run to cases whose id starts with a prefix.";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        [] | ["help"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        ["parity", rest @ ..] => parity(rest),
        ["smoke"] => smoke(),
        ["bench", rest @ ..] => bench::run(rest),
        ["gen-shell", rest @ ..] => gen_shell::run(rest),
        ["trace", rest @ ..] => trace::run(rest),
        ["nvm-diff", rest @ ..] => nvm_diff::run(rest),
        ["string-audit", rest @ ..] => string_audit::run(rest),
        [other, ..] => {
            eprintln!("xtask: unknown task `{other}`\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn parity(args: &[&str]) -> ExitCode {
    let record = args.contains(&"--record");
    let test = if args.contains(&"determinism") {
        "nvm_is_deterministic"
    } else if args.contains(&"launches") {
        "everyday_commands_launch_only_nvmimic"
    } else if args.contains(&"covers") {
        "record_covers"
    } else {
        "reference_cases_match_or_are_known"
    };

    if !cargo(&["build", "--quiet", "--package", "nvmimic-cli"]) {
        return ExitCode::FAILURE;
    }
    let compares = test == "reference_cases_match_or_are_known";
    let mut only = None;
    if compares && args.contains(&"--changed") {
        match changed::affected() {
            changed::Affected::Everything(why) => eprintln!("running every case: {why}"),
            changed::Affected::Cases(ids) if ids.is_empty() => {
                eprintln!("no case can be affected, nothing to run");
                return ExitCode::SUCCESS;
            }
            changed::Affected::Cases(ids) => {
                let file = repo_root().join("target/parity-only.txt");
                let ids: Vec<String> = ids.into_iter().collect();
                std::fs::write(&file, ids.join("\n") + "\n").unwrap();
                only = Some(file);
            }
        }
    }
    let network = compares && args.contains(&"--network");
    let run = if record || network || args.contains(&"--full") {
        "full"
    } else if args.contains(&"--all-shells") {
        "all-shells"
    } else {
        "everyday"
    };
    let ok = cargo_with(
        &[
            "test",
            "--quiet",
            "--package",
            "nvmimic-parity",
            "--test",
            "parity",
            test,
            "--",
            "--ignored",
            "--exact",
            "--nocapture",
        ],
        record,
        run,
        only.as_deref(),
        network,
    );
    // A run narrowed by hand says nothing about the cases it left out, and
    // neither does a network pass.
    if ok && compares && !network && env::var_os("NVMIMIC_CASES").is_none() {
        changed::mark_passed();
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// A minute or two with real Node: `nvmimic self install` into a throwaway
/// home, three real Node releases, and the everyday commands under each
/// shell. The archives are downloaded once and kept in
/// `target/parity-network/`.
fn smoke() -> ExitCode {
    if !cargo(&["build", "--quiet", "--package", "nvmimic-cli"]) {
        return ExitCode::FAILURE;
    }
    let ok = cargo_with(
        &[
            "test",
            "--quiet",
            "--package",
            "nvmimic-parity",
            "--test",
            "smoke",
            "--",
            "--ignored",
            "--nocapture",
        ],
        false,
        "full",
        None,
        true,
    );
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn cargo(args: &[&str]) -> bool {
    cargo_with(args, false, "full", None, false)
}

fn cargo_with(args: &[&str], record: bool, run: &str, only: Option<&Path>, network: bool) -> bool {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut command = Command::new(cargo);
    command
        .args(args)
        .current_dir(repo_root())
        .env("NVMIMIC_PARITY_RUN", run);
    if record {
        command.env("NVMIMIC_PARITY_RECORD", "1");
    }
    match only {
        Some(file) => command.env("NVMIMIC_PARITY_ONLY", file),
        None => command.env_remove("NVMIMIC_PARITY_ONLY"),
    };
    match network {
        true => command.env("NVMIMIC_PARITY_NETWORK", "1"),
        false => command.env_remove("NVMIMIC_PARITY_NETWORK"),
    };
    command.status().is_ok_and(|status| status.success())
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn reference_dir() -> PathBuf {
    repo_root().join(format!("tests/reference/nvm-{}", nvmimic::pin::TAG))
}
