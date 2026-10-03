//! Project tasks: `cargo xtask <task>`.

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let task = env::args().nth(1);
    match task.as_deref() {
        None | Some("help") => {
            println!("usage: cargo xtask <task>");
            println!();
            println!("tasks:");
            println!("  help    show this list");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("xtask: unknown task `{other}`");
            ExitCode::FAILURE
        }
    }
}
