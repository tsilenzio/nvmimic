//! `cargo xtask nvm-diff <old> <new>`: what changed in nvm between two
//! revisions, and which parts of nvmimic and which parity cases that touches.
//! It reads a clone of nvm-sh/nvm given by `--repo <path>` or `$NVM_REPO`.

use crate::nvmsh::{self, Unit};
use crate::trace::Map;
use std::collections::BTreeSet;
use std::process::{Command, ExitCode};

const FILES: &[&str] = &["nvm.sh", "nvm-exec", "bash_completion", "install.sh"];

pub fn run(args: &[&str]) -> ExitCode {
    let mut revisions = Vec::new();
    let mut repo = std::env::var("NVM_REPO").ok();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match *arg {
            "--repo" => repo = iter.next().map(|path| path.to_string()),
            other => revisions.push(other.to_string()),
        }
    }
    let (Some(repo), [old, new]) = (repo, revisions.as_slice()) else {
        eprintln!("usage: cargo xtask nvm-diff <old> <new> --repo <nvm clone>  (or NVM_REPO=...)");
        return ExitCode::FAILURE;
    };
    match report(&repo, old, new) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("nvm-diff: {message}");
            ExitCode::FAILURE
        }
    }
}

fn git(repo: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Lines a `git diff -U0` hunk header names: `(old_start, old_len, new_start, new_len)`.
fn hunk(header: &str) -> Option<(usize, usize, usize, usize)> {
    let inner = header.strip_prefix("@@ -")?;
    let (old, rest) = inner.split_once(" +")?;
    let new = rest.split(' ').next()?;
    let range = |text: &str| -> Option<(usize, usize)> {
        match text.split_once(',') {
            Some((start, len)) => Some((start.parse().ok()?, len.parse().ok()?)),
            None => Some((text.parse().ok()?, 1)),
        }
    };
    let (a, b) = range(old)?;
    let (c, d) = range(new)?;
    Some((a, b, c, d))
}

fn units_touched(units: &[Unit], start: usize, len: usize) -> Vec<String> {
    let lines: Vec<usize> = if len == 0 {
        vec![start.max(1)]
    } else {
        (start..start + len).collect()
    };
    let mut names: Vec<String> = lines
        .into_iter()
        .filter_map(|line| nvmsh::unit_at(units, line).map(|unit| unit.name.clone()))
        .collect();
    names.dedup();
    names
}

fn report(repo: &str, old: &str, new: &str) -> Result<String, String> {
    let map = Map::build()?;
    let mut out = format!("# nvm changes from `{old}` to `{new}`\n\n");

    let log = git(
        repo,
        &[
            "log",
            "--reverse",
            "--format=%h %s",
            &format!("{old}..{new}"),
        ],
    )?;
    out.push_str(&format!("## Commits ({})\n\n", log.lines().count()));
    for line in log.lines() {
        out.push_str(&format!("- {line}\n"));
    }

    let old_units = nvmsh::units(&git(repo, &["show", &format!("{old}:nvm.sh")])?);
    let new_units = nvmsh::units(&git(repo, &["show", &format!("{new}:nvm.sh")])?);
    let names = |units: &[Unit]| -> BTreeSet<String> {
        units.iter().map(|unit| unit.name.clone()).collect()
    };
    let (old_names, new_names) = (names(&old_units), names(&new_units));
    out.push_str("\n## Functions and dispatcher arms\n\n");
    for name in new_names.difference(&old_names) {
        out.push_str(&format!("- added: `{name}`\n"));
    }
    for name in old_names.difference(&new_names) {
        out.push_str(&format!("- removed: `{name}`\n"));
    }

    let mut touched: BTreeSet<String> = BTreeSet::new();
    let mut observable: Vec<String> = Vec::new();
    for file in FILES {
        let diff = git(repo, &["diff", "-U0", old, new, "--", file])?;
        if diff.is_empty() {
            continue;
        }
        for line in diff.lines() {
            if line.starts_with("@@") && *file == "nvm.sh" {
                if let Some((a, b, c, d)) = hunk(line) {
                    touched.extend(units_touched(&old_units, a, b));
                    touched.extend(units_touched(&new_units, c, d));
                }
            } else if (line.starts_with('+') || line.starts_with('-'))
                && !line.starts_with("+++")
                && !line.starts_with("---")
                && is_observable(line)
            {
                observable.push(format!("{file}: {line}"));
            }
        }
        if *file != "nvm.sh" {
            touched.insert(file.to_string());
        }
    }

    out.push_str("\n## Touched, with nvmimic's port and covering cases\n\n");
    out.push_str("| nvm unit | nvmimic | cases |\n|---|---|---|\n");
    for name in &touched {
        let port = map.anchors.get(name).map_or_else(
            || {
                if crate::gen_shell::is_verbatim(name) {
                    "shell/nvm.sh (copied, regenerate)".to_string()
                } else {
                    "real nvm (fallback)".to_string()
                }
            },
            |places| places.join(", "),
        );
        let cases = map
            .covers
            .get(name)
            .map_or(String::new(), |ids| ids.join(" "));
        out.push_str(&format!("| `{name}` | {port} | {cases} |\n"));
    }

    out.push_str(&format!(
        "\n## Message, exit status and environment lines ({})\n\n",
        observable.len()
    ));
    for line in &observable {
        out.push_str(&format!("    {line}\n"));
    }

    let tests = git(repo, &["diff", "--name-status", old, new, "--", "test/"])?;
    out.push_str(&format!(
        "\n## nvm's tests added or changed ({})\n\n",
        tests.lines().count()
    ));
    for line in tests.lines() {
        out.push_str(&format!("- {}\n", line.replace('\t', " ")));
    }
    Ok(out)
}

/// A changed line that can change what a script sees: printed text, an exit
/// status, the environment, help text or the unload list.
fn is_observable(line: &str) -> bool {
    let body = line[1..].trim();
    [
        "nvm_err",
        "nvm_echo",
        "printf",
        "return ",
        "exit ",
        "export ",
        "unset ",
        "EXIT_CODE=",
    ]
    .iter()
    .any(|marker| body.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hunk_headers() {
        assert_eq!(hunk("@@ -10,2 +12,3 @@ nvm_ls()"), Some((10, 2, 12, 3)));
        assert_eq!(hunk("@@ -5 +5,0 @@"), Some((5, 1, 5, 0)));
    }
}
