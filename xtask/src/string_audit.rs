//! `cargo xtask string-audit`: every fixed piece of text in
//! `crates/nvmimic/src/text/` must still appear in the pinned nvm.sh, so a
//! message nvm reworded fails here instead of drifting quietly.
//!
//! A string preceded by a `// string-audit: skip` comment, for text printed
//! by another program, is left out.

use std::fs;
use std::process::ExitCode;

pub fn run(_args: &[&str]) -> ExitCode {
    let reference = match fs::read_to_string(crate::reference_dir().join("nvm.sh")) {
        Ok(text) => unescape_shell(&text),
        Err(error) => {
            eprintln!("string-audit: {error}");
            return ExitCode::FAILURE;
        }
    };
    let dir = crate::repo_root().join("crates/nvmimic/src/text");
    let mut checked = 0;
    let mut missing = Vec::new();
    let mut files: Vec<_> = fs::read_dir(&dir)
        .map(|d| d.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    files.sort();
    for file in files {
        let text = fs::read_to_string(&file).unwrap_or_default();
        for (line, literal) in literals(&text) {
            for fragment in fragments(&literal) {
                checked += 1;
                if !reference.contains(&fragment) {
                    missing.push(format!("{}:{line}: {fragment:?}", file.display()));
                }
            }
        }
    }
    if missing.is_empty() {
        println!("string-audit: {checked} fragments found in nvm.sh");
        ExitCode::SUCCESS
    } else {
        for entry in &missing {
            eprintln!("not in nvm.sh: {entry}");
        }
        ExitCode::FAILURE
    }
}

/// nvm.sh with the backslash escapes of its double-quoted strings undone,
/// so `\"`, `\$` and `` \` `` read as the characters they print.
fn unescape_shell(text: &str) -> String {
    text.replace("\\\"", "\"")
        .replace("\\$", "$")
        .replace("\\`", "`")
}

/// String literals in Rust source with their line numbers, skipping any
/// after a `// string-audit: skip` comment until the next blank line.
fn literals(source: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    let mut skipping = false;
    let mut chars = source.char_indices().peekable();
    let line_of = |at: usize| source[..at].matches('\n').count() + 1;
    while let Some((at, c)) = chars.next() {
        match c {
            '/' if source[at..].starts_with("//") => {
                let end = source[at..].find('\n').map_or(source.len(), |n| at + n);
                if source[at..end].contains("string-audit: skip") {
                    skipping = true;
                }
                while chars.peek().is_some_and(|(i, _)| *i < end) {
                    chars.next();
                }
            }
            '\n' if source[at + 1..].starts_with('\n') => skipping = false,
            '"' => {
                let mut literal = String::new();
                while let Some((_, c)) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => match chars.next() {
                            Some((_, 'n')) => literal.push('\n'),
                            Some((_, 't')) => literal.push('\t'),
                            Some((_, '\n')) => {
                                while chars.peek().is_some_and(|(_, c)| c.is_whitespace()) {
                                    chars.next();
                                }
                            }
                            Some((_, other)) => literal.push(other),
                            None => {}
                        },
                        other => literal.push(other),
                    }
                }
                if !skipping {
                    found.push((line_of(at), literal));
                }
            }
            _ => {}
        }
    }
    found
}

/// The fixed text of a literal: the parts between `{placeholders}`, with
/// `{{` and `}}` read as braces, each one line at most and long enough to
/// mean something.
fn fragments(literal: &str) -> Vec<String> {
    let mut pieces = Vec::new();
    let mut current = String::new();
    let mut chars = literal.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                current.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                current.push('}');
            }
            '{' => {
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                }
                pieces.push(std::mem::take(&mut current));
            }
            _ => current.push(c),
        }
    }
    pieces.push(current);
    pieces
        .into_iter()
        .flat_map(|piece| piece.split('\n').map(str::to_string).collect::<Vec<_>>())
        .map(|piece| piece.trim().to_string())
        .filter(|piece| piece.chars().filter(|c| c.is_alphanumeric()).count() >= 3)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders_split_fragments() {
        assert_eq!(
            fragments("N/A: version \"{v}\" is not yet installed."),
            vec!["N/A: version \"", "\" is not yet installed."]
        );
        assert_eq!(
            fragments("Could not find {d}/*/bin in ${{PATH}}"),
            vec!["Could not find", "/*/bin in ${PATH}"]
        );
    }
}
