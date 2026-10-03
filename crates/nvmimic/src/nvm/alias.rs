//! Aliases: reading and writing alias files, following alias chains, the
//! built-in `node`/`stable`/`unstable`/`iojs` aliases, and listing them all.

use super::colors::{self, printf_b};
use super::local;
use super::version::{self, IOJS_PREFIX, NODE_PREFIX};
use crate::backend::AliasFile;
use crate::context::Nvm;
use crate::text::messages;

/// Turns `lts/-N` into the Nth-newest LTS alias and rejects uppercase LTS
/// names. On error the message has already gone to stderr.
///
/// nvm: nvm_normalize_lts
pub fn normalize_lts(nvm: &mut Nvm, lts: &str) -> Result<String, i32> {
    if let Some(n) = lts.strip_prefix("lts/-")
        && n.starts_with(|c: char| ('1'..='9').contains(&c))
        && (n.len() == 1 || n[1..].starts_with(|c: char| c.is_ascii_digit()))
    {
        // `cut -d '-' -f 2` keeps the digits up to the next dash.
        let digits = n.split('-').next().unwrap_or("");
        let count: i64 = shell_arithmetic(digits) + 1;
        let lts_dir = format!("{}/lts", nvm.backend.alias_dir(&nvm.vars));
        let names = ls_names(nvm, &lts_dir);
        let skip = names.len().saturating_sub(count.max(0) as usize);
        let result = if count <= 0 {
            String::new()
        } else {
            names.get(skip).cloned().unwrap_or_default()
        };
        if result != "*" {
            return Ok(format!("lts/{result}"));
        }
        nvm.io.err(messages::TOO_MANY_LTS);
        return Err(2);
    }
    if lts.starts_with("lts/") && lts != lts.to_ascii_lowercase() {
        nvm.io.err(messages::LTS_LOWERCASE);
        return Err(3);
    }
    Ok(lts.to_string())
}

/// `$((N+1))` for the digits nvm cut out; anything unparseable counts as 0.
fn shell_arithmetic(digits: &str) -> i64 {
    digits.parse().unwrap_or(0)
}

/// The names `command ls dir` prints, sorted, or an error on stderr.
fn ls_names(nvm: &mut Nvm, dir: &str) -> Vec<String> {
    match std::fs::read_dir(dir) {
        Ok(read) => {
            let mut names: Vec<String> = read
                .flatten()
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|name| !name.starts_with('.'))
                .collect();
            names.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
            names
        }
        Err(_) => {
            nvm.io.err(&messages::ls_missing(dir));
            Vec::new()
        }
    }
}

/// Lines of an alias file with `#` comments and trailing whitespace removed
/// and blank lines skipped.
///
/// nvm: nvm_print_alias_file
fn alias_file_lines(contents: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut rest = contents;
    while !rest.is_empty() {
        let (line, after) = match rest.find('\n') {
            Some(at) => (&rest[..at], &rest[at + 1..]),
            None => (rest, ""),
        };
        rest = after;
        let line = line.split('#').next().unwrap_or("");
        let is_space = |c: char| matches!(c, ' ' | '\t' | '\n' | '\u{0b}' | '\u{0c}' | '\r');
        if line.chars().all(is_space) {
            continue;
        }
        lines.push(line.trim_end_matches(is_space).to_string());
    }
    lines
}

/// The contents of one alias, and nvm's status: 1 for no name, 2 for a
/// missing alias, 3 for an invalid name. Messages go to stderr.
///
/// nvm: nvm_alias
pub fn alias(nvm: &mut Nvm, name: &str) -> (String, i32) {
    if name.is_empty() {
        nvm.io.err(messages::ALIAS_REQUIRED);
        return (String::new(), 1);
    }
    // nvm reads the status of `! ALIAS="$(...)"`, which is always 0.
    let Ok(name) = normalize_lts(nvm, name) else {
        return (String::new(), 0);
    };
    if name.is_empty() {
        return (String::new(), 2);
    }
    if format!("/{name}/").contains("/../") {
        nvm.io.err(&messages::invalid_alias_name(&name));
        return (String::new(), 3);
    }
    match nvm.backend.read_alias(&nvm.vars, &name) {
        AliasFile::Missing => {
            nvm.io.err(messages::ALIAS_MISSING);
            (String::new(), 2)
        }
        AliasFile::Unreadable(path) => {
            nvm.io.err(&messages::alias_unreadable(&path));
            (String::new(), 0)
        }
        AliasFile::Contents(contents) => (alias_file_lines(&contents).join("\n"), 0),
    }
}

/// Follows an alias chain to its end. `∞` means the chain loops. For a
/// built-in alias with no file, the locally resolved version is printed and
/// the status is 2.
///
/// nvm: nvm_resolve_alias
pub fn resolve_alias(nvm: &mut Nvm, pattern: &str) -> (String, i32) {
    if pattern.is_empty() {
        return (String::new(), 1);
    }
    let mut current = pattern.to_string();
    let mut seen = vec![current.clone()];
    loop {
        let (output, status) = nvm.quiet_err(|nvm| alias(nvm, &current));
        let output = if status == 0 { output } else { String::new() };
        let next = output.split('\n').next().unwrap_or("").to_string();
        if next.is_empty() {
            break;
        }
        if seen.contains(&next) {
            current = "∞".to_string();
            break;
        }
        seen.push(next.clone());
        current = next;
    }

    if !current.is_empty() && current != pattern {
        return match current.as_str() {
            "∞" | IOJS_PREFIX | "iojs-" | NODE_PREFIX => (current, 0),
            _ => (version::ensure_prefix(&current), 0),
        };
    }

    if version::is_implicit_alias(pattern) {
        let implicit = nvm.quiet_err(|nvm| print_implicit_alias_local(nvm, pattern).0);
        if !implicit.is_empty() {
            return (version::ensure_prefix(&implicit), 2);
        }
    }
    (String::new(), 2)
}

/// nvm: nvm_resolve_local_alias
pub fn resolve_local_alias(nvm: &mut Nvm, name: &str) -> (String, i32) {
    if name.is_empty() {
        return (String::new(), 1);
    }
    let (resolved, status) = resolve_alias(nvm, name);
    if resolved.is_empty() {
        return (String::new(), status);
    }
    if resolved == "∞" {
        return (resolved, 0);
    }
    local::version(nvm, &resolved)
}

/// The local value of a built-in alias: `stable`, `unstable`, `node` or `iojs`.
///
/// nvm: nvm_print_implicit_alias
pub fn print_implicit_alias_local(nvm: &mut Nvm, implicit: &str) -> (String, i32) {
    if !version::is_implicit_alias(implicit) {
        nvm.io.err(&messages::only_implicit_aliases());
        return (String::new(), 2);
    }
    match implicit {
        IOJS_PREFIX => {
            let (listing, status) = local::ls(nvm, IOJS_PREFIX);
            let mut found = listing.clone();
            if status == 0 {
                found = minors(
                    listing
                        .lines()
                        .map(|line| line.strip_prefix("iojs-").unwrap_or(line)),
                )
                .last()
                .cloned()
                .unwrap_or_default();
            }
            let out = if found == "N/A" {
                "N/A".to_string()
            } else {
                version::add_iojs_prefix(&found)
            };
            (out, status)
        }
        NODE_PREFIX => ("stable".to_string(), 0),
        _ => {
            let (listing, _) = local::ls(nvm, NODE_PREFIX);
            let mut stable = String::new();
            let mut unstable = String::new();
            for minor in minors(listing.lines()) {
                let normalized = version::normalize(&minor);
                if !normalized.starts_with('0') {
                    stable = minor;
                } else {
                    let number: f64 = normalized.parse().unwrap_or(0.0);
                    match ((number / 1_000_000.0).floor() as i64) % 2 {
                        0 => stable = minor,
                        _ => unstable = minor,
                    }
                }
            }
            if implicit == "stable" {
                (stable, 0)
            } else if unstable.is_empty() {
                ("N/A".to_string(), 0)
            } else {
                (unstable, 0)
            }
        }
    }
}

/// `grep -e '^v' | cut -c2- | cut -d . -f 1,2 | uniq`.
fn minors<'s>(lines: impl Iterator<Item = &'s str>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in lines.filter(|line| line.starts_with('v')) {
        let minor = line[1..].split('.').take(2).collect::<Vec<_>>().join(".");
        if out.last() != Some(&minor) {
            out.push(minor);
        }
    }
    out
}

/// Writes an alias file, or prints why not.
///
/// nvm: nvm_make_alias
pub fn make_alias(nvm: &mut Nvm, name: &str, target: &str) -> i32 {
    if name.is_empty() {
        nvm.io.err(messages::ALIAS_NAME_REQUIRED);
        return 1;
    }
    if target.is_empty() {
        nvm.io.err(messages::ALIAS_TARGET_REQUIRED);
        return 2;
    }
    if format!("/{name}/").contains("/../") {
        nvm.io.err(&messages::invalid_alias_name(name));
        return 3;
    }
    match nvm.backend.write_alias(&nvm.vars, name, target) {
        Ok(()) => 0,
        Err(error) => {
            let path = format!("{}/{name}", nvm.backend.alias_dir(&nvm.vars));
            nvm.io.err(&messages::tee_failed(&path, &error));
            1
        }
    }
}

/// How one alias line is drawn, from the variables nvm passes along.
#[derive(Clone, Copy, Default)]
pub struct AliasStyle {
    pub default: bool,
    pub lts: bool,
}

/// One `name -> target (-> version)` line, colored when colors are on.
///
/// nvm: nvm_print_formatted_alias
pub fn print_formatted_alias(
    nvm: &mut Nvm,
    name: &str,
    dest: &str,
    version: Option<&str>,
    style: AliasStyle,
    current: &str,
) {
    let version = match version.filter(|v| !v.is_empty()) {
        Some(v) => v.to_string(),
        None => local::version(nvm, dest).0,
    };
    let installed_color = colors::get_color(nvm, 1);
    let _system_color = colors::get_color(nvm, 2);
    let current_color = colors::get_color(nvm, 3);
    let not_installed_color = colors::get_color(nvm, 4);
    let default_color = colors::get_color(nvm, 5);
    let lts_color = colors::get_color(nvm, 6);

    let mut alias_format = "%s".to_string();
    let mut dest_format = "%s".to_string();
    let mut version_format = "%s".to_string();
    let mut newline = if style.default {
        " (default)\\n".to_string()
    } else {
        "\\n".to_string()
    };
    let mut arrow = "->".to_string();
    let colors_on = nvm.vars.get("NVM_HAS_COLORS") == "1" || colors::has_colors(nvm);
    if colors_on {
        arrow = "\\033[0;90m->\\033[0m".to_string();
        if style.default {
            newline = format!(" \\033[{default_color}(default)\\033[0m\\n");
        }
        let paint = |color: &str| format!("\\033[{color}%s\\033[0m");
        if version == current {
            alias_format = paint(&current_color);
            dest_format = paint(&current_color);
            version_format = paint(&current_color);
        } else if local::is_version_installed(nvm, &version) {
            alias_format = paint(&installed_color);
            dest_format = paint(&installed_color);
            version_format = paint(&installed_color);
        } else if version == "∞" || version == "N/A" {
            alias_format = paint(&not_installed_color);
            dest_format = paint(&not_installed_color);
            version_format = paint(&not_installed_color);
        }
        if style.lts {
            alias_format = paint(&lts_color);
        }
        // `${DEST%/*}` is everything before the last slash, or all of it.
        if dest.rsplit_once('/').map_or(dest, |(head, _)| head) == "lts" {
            dest_format = paint(&lts_color);
        }
    } else if version != "∞" && version != "N/A" {
        version_format = "%s *".to_string();
    }

    let line = if dest == version {
        printf(
            &format!("{alias_format} {arrow} {version_format}{newline}"),
            &[name, dest],
        )
    } else {
        printf(
            &format!("{alias_format} {arrow} {dest_format} ({arrow} {version_format}){newline}"),
            &[name, dest, &version],
        )
    };
    nvm.io.out_raw(&line);
}

/// `printf -- format args...` for formats holding only `%s` and backslash
/// escapes, which is all nvm's alias lines use.
fn printf(format: &str, args: &[&str]) -> String {
    let mut out = String::new();
    let mut args = args.iter();
    let mut rest = format;
    while let Some(at) = rest.find("%s") {
        out.push_str(&printf_b(&rest[..at]));
        out.push_str(args.next().copied().unwrap_or(""));
        rest = &rest[at + 2..];
    }
    out.push_str(&printf_b(rest));
    out
}

/// nvm: nvm_print_alias_path
fn print_alias_path(nvm: &mut Nvm, alias_dir: &str, path: &str, style: AliasStyle, current: &str) {
    let name = path
        .strip_prefix(&format!("{alias_dir}/"))
        .unwrap_or(path)
        .to_string();
    let dest = nvm.quiet_err(|nvm| alias(nvm, &name)).0;
    if !dest.is_empty() {
        print_formatted_alias(nvm, &name, &dest, None, style, current);
    }
}

/// nvm: nvm_print_default_alias
fn print_default_alias(nvm: &mut Nvm, name: &str, current: &str) {
    let dest = print_implicit_alias_local(nvm, name).0;
    if !dest.is_empty() {
        let style = AliasStyle {
            default: true,
            lts: false,
        };
        print_formatted_alias(nvm, name, &dest, None, style, current);
    }
}

/// Every alias starting with `prefix`: user aliases, then the built-in ones,
/// then the LTS aliases, each group sorted.
///
/// nvm: nvm_list_aliases
pub fn list_aliases(nvm: &mut Nvm, prefix: &str) -> i32 {
    let current = local::ls_current(nvm);
    let alias_dir = nvm.backend.alias_dir(&nvm.vars);
    nvm.backend.make_dirs(&format!("{alias_dir}/lts"));

    if prefix.starts_with("lts/") {
        let (output, status) = alias(nvm, prefix);
        if !output.is_empty() {
            nvm.io.echo(&output);
        }
        return status;
    }

    let has_colors = colors::has_colors(nvm);
    let flag = if has_colors { "1" } else { "0" };

    let user = nvm.backend.alias_glob(&nvm.vars, prefix);
    let lines = group(nvm, flag, |nvm| {
        for path in &user {
            print_alias_path(nvm, &alias_dir, path, AliasStyle::default(), &current);
        }
    });
    emit_sorted(nvm, lines);

    let lines = group(nvm, flag, |nvm| {
        for name in [NODE_PREFIX, "stable", "unstable", IOJS_PREFIX] {
            if !nvm.backend.alias_exists(&nvm.vars, name) && (prefix.is_empty() || name == prefix) {
                print_default_alias(nvm, name, &current);
            }
        }
    });
    emit_sorted(nvm, lines);

    let lts = nvm.backend.alias_glob(&nvm.vars, &format!("lts/{prefix}"));
    let lines = group(nvm, flag, |nvm| {
        let style = AliasStyle {
            default: false,
            lts: true,
        };
        for path in &lts {
            let line = nvm.capture(|nvm| print_alias_path(nvm, &alias_dir, path, style, &current));
            if !line.is_empty() {
                nvm.io.echo(&line);
            }
        }
    });
    emit_sorted(nvm, lines);
    0
}

/// Runs one of nvm's background-job groups, whose combined output is piped
/// through `sort`. Colors were decided once beforehand and passed down.
fn group(nvm: &mut Nvm, has_colors: &str, body: impl FnOnce(&mut Nvm<'_>)) -> Vec<String> {
    let flag = has_colors.to_string();
    let output = nvm.capture(|inner| {
        inner.vars.set_quiet("NVM_HAS_COLORS", &flag);
        body(inner);
    });
    if output.is_empty() {
        Vec::new()
    } else {
        output.split('\n').map(str::to_string).collect()
    }
}

fn emit_sorted(nvm: &mut Nvm, mut lines: Vec<String>) {
    lines.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    for line in lines {
        nvm.io.echo(&line);
    }
}
