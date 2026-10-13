//! `shelfy`: a whole program built on the nvmimic library, short enough to
//! read in one sitting.
//!
//! It does nvm's job like nvmimic does, with two things of its own: Node
//! versions live on a `Shelf` (see `tests/common/shelf.rs`) and not in
//! nvm's folders, and everything that nvmimic names after itself is named
//! `shelfy`. The library does the rest. A program like this has three jobs:
//!
//! 1. Answer the shell functions. They start the program with
//!    [`HANDOFF`] as the first argument, and [`Host::handoff`] does the work.
//! 2. Print the shell code a new shell loads, for `shelfy init`.
//! 3. Set itself up and remove itself, for `shelfy self ...`.
//!
//! `crates/nvmimic/tests/other_store.rs` runs this program in real shells.

#[path = "../tests/common/shelf.rs"]
mod shelf;

use nvmimic::host::{HANDOFF, Host, is_tty};
use nvmimic::init::Named;
use nvmimic::install::{self, Env, Options, TerminalUi};
use shelf::{SHELFY, Shelf, ShelfSteps};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    // The shelf stands in the home folder unless SHELFY_ROOT moves it.
    let root = env::var_os("SHELFY_ROOT")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default();
    let shelf = Shelf::at(&root);
    let host = Host {
        identity: &SHELFY,
        backend: &shelf,
        // Shell text of shelfy's own, which every loading shell runs.
        init_extra: "SHELFY_LOADED=1\n",
    };

    match words.as_slice() {
        [first, function, rest @ ..] if *first == HANDOFF => {
            let rest: Vec<String> = rest.iter().map(|arg| arg.to_string()).collect();
            // `handoff` alone would do. This form adds shell text of
            // shelfy's own to what the calling shell runs after the call.
            ExitCode::from(host.handoff_with(function, &rest, |outcome| {
                format!("SHELFY_LAST_STATUS={}\n", outcome.status)
            }))
        }
        ["--version"] => {
            println!("{}", SHELFY.version);
            ExitCode::SUCCESS
        }
        ["init"] => init(&host, None),
        ["init", shell] => match Named::parse(shell) {
            Some(named) => init(&host, Some(named)),
            None => fail(&format!("init doesn't know the shell {shell}.")),
        },
        ["self", action, flags @ ..] => self_command(&shelf, action, flags),
        _ => fail("usage: shelfy init [shell] | shelfy self <command> | shelfy --version"),
    }
}

fn init(host: &Host, named: Option<Named>) -> ExitCode {
    match host.init(named) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(message) => fail(&message),
    }
}

/// `shelfy self install`, `eject`, `uninstall`, `disable` and `enable`. The
/// library does each of them for the identity in `Env`, and `ShelfSteps`
/// adds what only shelfy knows how to do.
fn self_command(shelf: &Shelf, action: &str, flags: &[&str]) -> ExitCode {
    let options = Options {
        force: flags.contains(&"--force"),
        dry_run: flags.contains(&"--dry-run"),
        purge: flags.contains(&"--purge"),
        terminal: is_tty(0),
    };
    let env = Env::from_process_for(&SHELFY);
    let mut ui = TerminalUi;
    let steps = ShelfSteps(shelf);
    let result = match action {
        "install" => match env::current_exe() {
            Ok(program) => install::self_install_with(&env, &program, &options, &mut ui, &steps),
            Err(error) => Err(error.to_string()),
        },
        "eject" => install::self_eject_with(&env, &options, &mut ui, &steps),
        // Hands `nvm` to nvmimic or another sister tool if one is set up,
        // and to official nvm otherwise. `enable` takes it back.
        "disable" => install::self_disable_with(&env, &options, &mut ui, &steps),
        "enable" => match env::current_exe() {
            Ok(program) => install::self_enable_with(&env, &program, &options, &mut ui, &steps),
            Err(error) => Err(error.to_string()),
        },
        "uninstall" => install::self_uninstall_with(&env, &options, &mut ui, &steps),
        _ => Err(format!("there is no command self {action}.")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => fail(&message),
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("shelfy: {message}");
    ExitCode::FAILURE
}
