//! Project tasks: `cargo xtask <task>`.

mod bench;

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "\
usage: cargo xtask <task>

tasks:
  parity [--record]                 compare nvmimic with real nvm on every case
  parity determinism [--record]     run real nvm against itself on every case
  bench [--check] [--runs=N]        time sourcing nvm.sh for real nvm and nvmimic
  help                              show this list

--record rewrites cases/KNOWN_FAILING or cases/NONDETERMINISTIC from the run.
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
        ["bench", rest @ ..] => bench::run(rest),
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
    } else {
        "reference_cases_match_or_are_known"
    };

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
            "parity",
            test,
            "--",
            "--ignored",
            "--exact",
            "--nocapture",
        ],
        record,
    );
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn cargo(args: &[&str]) -> bool {
    cargo_with(args, false)
}

fn cargo_with(args: &[&str], record: bool) -> bool {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut command = Command::new(cargo);
    command.args(args).current_dir(repo_root());
    if record {
        command.env("NVMIMIC_PARITY_RECORD", "1");
    }
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
