//! A compiled stand-in for `node`, for tests that count program launches.
//!
//! The stand-in releases the parity cases install have a shell script as
//! their `node`, and nvmimic never remembers what a script answers. A test
//! that wants to see an answer remembered links this program in under the
//! name `node`. It reports the version named by the folder it was started
//! from, `<version>/bin/node`, and notes each launch in
//! `$NVMIMIC_LAUNCH_LOG` like the scripts do.

use std::io::Write;
use std::path::Path;

fn main() {
    let mut args = std::env::args();
    let started_as = args.next().unwrap_or_default();
    if let Some(log) = std::env::var_os("NVMIMIC_LAUNCH_LOG").filter(|log| !log.is_empty())
        && let Ok(mut log) = std::fs::OpenOptions::new().append(true).create(true).open(log)
    {
        let _ = writeln!(log, "{started_as}");
    }
    let version = Path::new(&started_as)
        .ancestors()
        .nth(2)
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    let rest: Vec<String> = args.collect();
    match rest.first().map(String::as_str) {
        Some("--version" | "-v") => println!("{version}"),
        _ => println!("node {version}: {}", rest.join(" ")),
    }
}
