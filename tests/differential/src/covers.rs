//! Which parts of nvm.sh each case runs.
//!
//! A case file lists them in `# covers:` headers, as the names
//! [`crate::nvmsh`] gives nvm.sh's functions and the arms of `nvm()`. The
//! lists are measured: the case runs against a copy of the pinned nvm.sh
//! that notes every function and arm it enters, from the moment the case's
//! body starts. `cargo xtask parity covers` does the measuring and writes
//! the headers. Tests from nvm's own suite have no header to write to, so
//! theirs go in `cases/NVM_SUITE_COVERS`.
//!
//! The lists say which cases a change to nvmimic can affect: the ones that
//! run the nvm function whose port was changed.

use crate::nvmsh;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// The variable naming the file a traced nvm.sh writes to.
pub const LOG_VAR: &str = "NVMIMIC_COVERS_LOG";

/// The file listing what each suite test runs: `<case id>: <names>`.
pub const SUITE_COVERS: &str = "NVM_SUITE_COVERS";

/// Header keys that come before the `# covers:` lines in a case file.
const HEADER_KEYS: &[&str] = &["id", "shells", "source", "setup", "mirror"];

/// nvm.sh with a line added at the start of every function and of every arm
/// of `nvm()`. The line appends the unit's name to the file `$LOG_VAR`
/// names, and does nothing while that variable is unset.
pub fn traced(source: &str) -> String {
    let starts: BTreeMap<usize, String> = nvmsh::units(source)
        .into_iter()
        .map(|unit| (unit.start, unit.name))
        .collect();
    let mut traced = String::with_capacity(source.len() * 2);
    for (index, line) in source.split('\n').enumerate() {
        if index > 0 {
            traced.push('\n');
        }
        traced.push_str(line);
        if let Some(name) = starts.get(&(index + 1)) {
            traced.push_str(&format!(
                "\n[ -z \"${{{LOG_VAR}-}}\" ] || printf '%s\\n' '{name}' >> \"${{{LOG_VAR}}}\""
            ));
        }
    }
    traced
}

/// What each suite test runs, by case id.
pub fn suite_covers() -> BTreeMap<String, Vec<String>> {
    fs::read_to_string(crate::cases::cases_dir().join(SUITE_COVERS))
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once(": "))
        .map(|(id, names)| {
            (
                id.to_string(),
                names.split_whitespace().map(str::to_string).collect(),
            )
        })
        .collect()
}

pub fn write_suite_covers(covers: &BTreeMap<String, BTreeSet<String>>) {
    let mut text = String::from(
        "# What each test from nvm's own suite runs in nvm.sh, measured.\n\
         # Regenerate with `cargo xtask parity covers`.\n",
    );
    for (id, names) in covers {
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        text.push_str(&format!("{id}: {}\n", names.join(" ")));
    }
    fs::write(crate::cases::cases_dir().join(SUITE_COVERS), text).unwrap();
}

/// Replaces a case file's `# covers:` headers with `names`, wrapped so no
/// line gets long. Returns whether the file changed.
pub fn write_header(file: &Path, names: &BTreeSet<String>) -> bool {
    let text = fs::read_to_string(file).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let header_end = lines
        .iter()
        .position(|line| !line.starts_with('#'))
        .unwrap_or(lines.len());
    let is_covers = |line: &str| line.starts_with("# covers: ");
    let is_key = |line: &str| {
        line.strip_prefix("# ")
            .and_then(|rest| rest.split_once(": "))
            .is_some_and(|(key, _)| HEADER_KEYS.contains(&key))
    };
    let kept: Vec<&str> = lines[..header_end]
        .iter()
        .copied()
        .filter(|line| !is_covers(line))
        .collect();
    let insert_at = kept
        .iter()
        .rposition(|line| is_key(line))
        .map_or(0, |index| index + 1);

    let mut covers: Vec<String> = Vec::new();
    for name in names {
        match covers.last_mut() {
            Some(line) if line.len() + 1 + name.len() <= 100 => {
                line.push(' ');
                line.push_str(name);
            }
            _ => covers.push(format!("# covers: {name}")),
        }
    }

    let mut out: Vec<&str> = kept[..insert_at].to_vec();
    out.extend(covers.iter().map(String::as_str));
    out.extend(&kept[insert_at..]);
    out.extend(&lines[header_end..]);
    let mut new_text = out.join("\n");
    new_text.push('\n');
    if new_text == text {
        return false;
    }
    fs::write(file, new_text).unwrap();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traces_functions_and_arms() {
        let source =
            "a_b() {\n  x\n}\nnvm() {\n  case $1 in\n    \"use\")\n      y\n    ;;\n  esac\n}\n";
        let traced = traced(source);
        let marks: Vec<&str> = traced
            .lines()
            .filter(|line| line.contains(LOG_VAR))
            .filter_map(|line| line.split('\'').nth(3))
            .collect();
        assert_eq!(marks, ["a_b", "nvm", "nvm:use"]);
        assert!(traced.ends_with("}\n"));
        assert!(traced.starts_with("a_b() {\n[ -z"));
    }
}
