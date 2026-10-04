//! `.nvmrc` files: finding one and reading the version out of it.

use super::colors;
use super::paths;
use crate::context::Nvm;
use crate::text::messages;

/// Whitespace as `[[:space:]]` matches it in the C locale.
fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{0b}' | '\u{0c}' | '\r')
}

fn trim(s: &str) -> &str {
    s.trim_matches(is_space)
}

/// The non-comment, non-blank lines of `.nvmrc` text, trimmed, as nvm's
/// `sed 's/#.*//' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | grep -v '^$'`
/// leaves them.
fn content_lines(content: &str) -> Vec<String> {
    content
        .split('\n')
        .map(|line| trim(line.split('#').next().unwrap_or("")).to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

/// The bare version in `.nvmrc` text, or `None` after printing nvm's
/// "invalid .nvmrc!" message to stderr.
///
/// nvm: nvm_process_nvmrc_content
pub fn process_content(nvm: &mut Nvm, content: &str) -> Option<String> {
    let lines = content_lines(content);
    let parsed = lines.join("\n");
    let mut keys: Vec<String> = Vec::new();
    let mut unpaired: Option<String> = None;

    for line in &lines {
        let before_equals = line.split('=').next().unwrap_or("");
        if before_equals.is_empty() || !line.contains('=') {
            if unpaired.is_some() {
                invalid(nvm, &parsed);
                return None;
            }
            unpaired = Some(line.clone());
            continue;
        }
        let key = trim(before_equals);
        if key == "node" || duplicate_key(key, &keys) {
            invalid(nvm, &parsed);
            return None;
        }
        keys.push(key.to_string());
    }

    match unpaired {
        Some(version) if !lines.is_empty() => Some(version),
        _ => {
            invalid(nvm, &parsed);
            None
        }
    }
}

/// nvm tests for a repeated key with `grep -E "(^| )${key}( |$)"` over the
/// keys seen so far, each preceded by a space, so the key is read as a pattern.
fn duplicate_key(key: &str, keys: &[String]) -> bool {
    let joined: String = keys.iter().map(|k| format!(" {k}")).collect();
    match super::posix_re::ere(&format!("(^| ){key}( |$)")) {
        Some(re) => re.is_match(&joined),
        None => false,
    }
}

/// nvm: nvm_nvmrc_invalid_msg
pub fn invalid(nvm: &mut Nvm, parsed: &str) {
    let error = colors::wrap_with_color_code(nvm, "r", messages::NVMRC_INVALID);
    let warn =
        colors::wrap_with_color_code(nvm, "y", &format!("{}\n{parsed}", messages::NVMRC_PARSED));
    // Each half is built in `$( ... )`, which drops trailing newlines.
    let error = error.trim_end_matches('\n');
    let warn = warn.trim_end_matches('\n');
    nvm.io.err(&format!("{error}\n\n{warn}"));
}

/// Reads `.nvmrc` from the working directory upward.
///
/// Returns the version and nvm's exit status: 1 when there is no file or it
/// is invalid, 2 when it holds nothing. The "Found ..." line goes to stdout
/// unless `NVM_SILENT=1`.
///
/// nvm: nvm_rc_version
pub fn rc_version(nvm: &mut Nvm, silent: bool) -> (Option<String>, i32) {
    let pwd = nvm.vars.get("PWD").to_string();
    let Some(path) = paths::find_nvmrc(&pwd) else {
        if !silent {
            nvm.io.err(messages::NO_VERSION_NO_NVMRC);
        }
        return (None, 1);
    };
    let content = std::fs::read(&path)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default();
    let content = content.trim_end_matches('\n');
    let Some(version) = process_content(nvm, content) else {
        return (None, 1);
    };
    if version.is_empty() {
        if !silent {
            nvm.io.err(&messages::empty_nvmrc(&path));
        }
        return (None, 2);
    }
    if !silent {
        nvm.io.echo(&messages::found_nvmrc(&path, &version));
    }
    (Some(version), 0)
}

/// Writes the resolved version to `$PWD/.nvmrc`. Returns nvm's status: 1
/// when the version does not resolve, 3 when the file cannot be written.
///
/// nvm: nvm_write_nvmrc
pub fn write_nvmrc(nvm: &mut Nvm, version: &str, silent: bool) -> i32 {
    let resolved = super::local::version(nvm, version).0;
    if resolved == "∞" || resolved == "N/A" {
        return 1;
    }
    let path = format!("{}/.nvmrc", nvm.vars.get("PWD"));
    if let Err(error) = std::fs::write(&path, format!("{resolved}\n")) {
        nvm.io.err(&messages::tee_failed(&path, &error));
        if !silent {
            nvm.io.err(&messages::unable_to_write_nvmrc(&resolved));
        }
        return 3;
    }
    if !silent {
        nvm.io.echo(&messages::wrote_nvmrc(&resolved));
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_drop_comments_and_blanks() {
        assert_eq!(
            content_lines("  22 # hi\n\n# x\na = b\r"),
            vec!["22", "a = b"]
        );
    }
}
