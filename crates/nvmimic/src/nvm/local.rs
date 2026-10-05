//! Installed versions: listing them, resolving a pattern to one, and finding
//! the active one. Node itself is asked for its version once per binary.

use super::alias;
use super::nvmrc;
use super::paths;
use super::posix_re;
use super::remembered::{self, Remembered};
use super::version::{self, IOJS_PREFIX, NODE_PREFIX};
use crate::backend::{Flavor, Flavors};
use crate::context::Nvm;
use std::cmp::Ordering;

/// The `node` (or `iojs`) that `command which` finds on `PATH`.
pub fn which_node(nvm: &Nvm, name: &str) -> Option<String> {
    paths::which(nvm.vars.get("PATH"), name)
}

/// The names a Node program goes by.
pub const NODE_NAMES: [&str; 2] = ["node", "iojs"];

/// What a Node binary prints for `--version`, or `None` when it cannot be
/// started. nvm runs the binary every time. nvmimic runs it once and
/// remembers the answer until the binary's file, `NODE_OPTIONS` or the
/// operating system build changes. A binary that fails is asked again on
/// every call, so one that cannot run on this machine is seen as nvm sees
/// it: as printing nothing. So is anything that is not a compiled program
/// named `node` or `iojs`, such as the stand-in a version manager puts on
/// `PATH`, which picks a different Node from one directory to the next.
pub fn binary_version(nvm: &Nvm, binary: &str) -> Option<String> {
    let remembered = std::fs::canonicalize(binary).ok().and_then(|resolved| {
        if !remembered::is_compiled(&resolved, &NODE_NAMES) {
            return None;
        }
        let subject = resolved.display().to_string();
        let key = [
            subject.clone(),
            remembered::fingerprint(&resolved)?,
            nvm.vars.get("NODE_OPTIONS").to_string(),
        ];
        Remembered::new(nvm, "node-version", &subject, &key)
    });
    if let Some(version) = remembered.as_ref().and_then(Remembered::read) {
        return Some(version);
    }
    let (output, version) = remembered::ask_version(nvm, binary)?;
    if let Some(remembered) = remembered {
        remembered.keep(&output, &version);
    }
    Some(version)
}

/// nvm: nvm_ls_current
pub fn ls_current(nvm: &Nvm) -> String {
    let Some(node_path) = which_node(nvm, "node") else {
        return "none".to_string();
    };
    match nvm.backend.owner_of(&nvm.vars, &node_path) {
        Some(Flavor::Iojs) => {
            let iojs = which_node(nvm, "iojs").and_then(|path| binary_version(nvm, &path));
            version::add_iojs_prefix(&iojs.unwrap_or_default())
        }
        Some(Flavor::Node) => match binary_version(nvm, &node_path) {
            Some(v) if v == "v0.6.21-pre" => "v0.6.21".to_string(),
            Some(v) if !v.is_empty() => v,
            _ => "none".to_string(),
        },
        None => "system".to_string(),
    }
}

/// `PATH` as `nvm deactivate` would leave it.
fn deactivated_path(nvm: &Nvm) -> String {
    nvm.backend
        .strip_path(&nvm.vars, nvm.vars.get("PATH"), "/bin")
}

/// nvm: nvm_has_system_node
pub fn has_system_node(nvm: &Nvm) -> bool {
    paths::on_path(&deactivated_path(nvm), "node")
}

/// nvm: nvm_has_system_iojs
pub fn has_system_iojs(nvm: &Nvm) -> bool {
    paths::on_path(&deactivated_path(nvm), "iojs")
}

/// What `node -v` prints for the system node, outside nvm's `PATH` entries.
pub fn system_node_version(nvm: &Nvm) -> String {
    paths::which(&deactivated_path(nvm), "node")
        .and_then(|node| binary_version(nvm, &node))
        .unwrap_or_default()
}

/// nvm: nvm_is_version_installed
pub fn is_version_installed(nvm: &Nvm, version: &str) -> bool {
    if version.is_empty() {
        return false;
    }
    match paths::version_path(nvm, version) {
        Ok(path) => nvm.backend.is_executable(&format!("{path}/bin/node")),
        Err(_) => false,
    }
}

/// The installed versions matching a pattern, one per line, and nvm's exit
/// status (3 when nothing matches, after printing `N/A`).
///
/// nvm: nvm_ls
pub fn ls(nvm: &mut Nvm, pattern: &str) -> (String, i32) {
    let mut pattern = pattern.to_string();
    if pattern.contains('#') || pattern.contains('\n') {
        match nvm.quiet_err(|nvm| nvmrc::process_content(nvm, &pattern)) {
            Some(parsed) => pattern = parsed,
            None => return ("N/A".to_string(), 3),
        }
    }
    if pattern == "current" {
        return (ls_current(nvm), 0);
    }

    let mut printed = String::new();
    if pattern == IOJS_PREFIX || pattern == NODE_PREFIX {
        pattern.push('-');
    } else {
        let target = nvm.quiet_err(|nvm| alias::resolve_alias(nvm, &pattern).0);
        if target == "system" && (has_system_iojs(nvm) || has_system_node(nvm)) {
            let system = system_node_version(nvm);
            return if system.is_empty() {
                ("system".to_string(), 0)
            } else {
                (format!("system {system}"), 0)
            };
        }
        // A failed lookup has already printed what it found, such as `N/A`
        // for an alias to a version that is not installed, and nvm carries on.
        let (output, status) = alias::resolve_local_alias(nvm, &pattern);
        if status == 0 {
            return (output, 0);
        }
        printed = output;
        pattern = version::ensure_prefix(&pattern);
    }
    let with_printed = |listing: String| -> String {
        match (printed.is_empty(), listing.is_empty()) {
            (true, _) => listing,
            (false, true) => printed.clone(),
            (false, false) => format!("{printed}\n{listing}"),
        }
    };
    if pattern == "N/A" {
        return (with_printed(String::new()), 0);
    }

    let mut versions = String::new();
    let mut add_system = false;
    if pattern.starts_with('v') && version::num_groups(&pattern) == 3 {
        if is_version_installed(nvm, &pattern) {
            versions = pattern.clone();
        } else if is_version_installed(nvm, &version::add_iojs_prefix(&pattern)) {
            versions = version::add_iojs_prefix(&pattern);
        }
    } else {
        if !matches!(pattern.as_str(), "iojs-" | "node-" | "system") {
            let groups = version::num_groups(&pattern);
            if groups == 1 || groups == 2 {
                pattern = format!("{}.", pattern.strip_suffix('.').unwrap_or(&pattern));
            }
        }

        let flavors;
        if version::is_iojs(&pattern) {
            flavors = Flavors::Iojs;
            pattern = version::strip_iojs_prefix(&pattern).to_string();
            add_system = has_system_iojs(nvm);
        } else if pattern == "node-" {
            flavors = Flavors::Node;
            pattern.clear();
            add_system = has_system_node(nvm);
        } else {
            flavors = Flavors::Both;
            add_system = has_system_iojs(nvm) || has_system_node(nvm);
        }

        let search = if pattern.is_empty() {
            pattern = "v".to_string();
            ".*".to_string()
        } else {
            pattern.replace('.', "\\.").replace('#', "\\#")
        };
        versions = list_installed(nvm, flavors, &search).join("\n");
    }

    if add_system {
        let system = system_node_version(nvm);
        let line = if system.is_empty() {
            "system".to_string()
        } else {
            format!("system {system}")
        };
        match pattern.as_str() {
            "" | "v" => versions = format!("{versions}\n{line}"),
            "system" => versions = line,
            _ => {}
        }
    }

    if versions.is_empty() {
        return (with_printed("N/A".to_string()), 3);
    }
    (with_printed(versions), 0)
}

/// nvm's `find | sed | sort -t. -u -k 1.2,1n -k 2,2n -k 3,3n | sed` over the
/// installed versions, with the same names going in and coming out.
fn list_installed(nvm: &Nvm, flavors: Flavors, search: &str) -> Vec<String> {
    let search_re = posix_re::bre(search);

    let mut lines: Vec<String> = Vec::new();
    for installed in nvm.backend.installed(&nvm.vars, flavors) {
        // nvm matches the pattern against `<flavor>/<version>` and then
        // sorts `<version>.<flavor>`.
        let flavor = installed.flavor.name();
        let line = format!("{flavor}/{}", installed.name);
        if !search_re.as_ref().is_some_and(|re| re.is_match(&line)) {
            continue;
        }
        lines.push(format!("{}.{flavor}", installed.name));
    }

    sort_versions(&mut lines);

    lines
        .into_iter()
        .map(|line| {
            let restored = match line.rfind('.') {
                Some(at) if at + 1 < line.len() => format!("{}-{}", &line[at + 1..], &line[..at]),
                _ => line,
            };
            restored
                .strip_prefix("node-")
                .map(str::to_string)
                .unwrap_or(restored)
        })
        .collect()
}

/// `sort -t. -u -k 1.2,1n -k 2,2n -k 3,3n`: numeric on the first field
/// without its first character, then the second and third fields. Lines
/// equal on all three keys are kept once.
pub(crate) fn sort_versions(lines: &mut Vec<String>) {
    fn keys(line: &str) -> [f64; 3] {
        let fields: Vec<&str> = line.split('.').collect();
        let first = fields.first().map_or("", |f| f.get(1..).unwrap_or(""));
        [
            numeric_prefix(first),
            numeric_prefix(fields.get(1).copied().unwrap_or("")),
            numeric_prefix(fields.get(2).copied().unwrap_or("")),
        ]
    }
    lines.sort_by(|a, b| keys(a).partial_cmp(&keys(b)).unwrap_or(Ordering::Equal));
    lines.dedup_by(|b, a| keys(a) == keys(b));
}

/// The number `sort -n` reads from the start of a field.
fn numeric_prefix(field: &str) -> f64 {
    let trimmed = field.trim_start_matches([' ', '\t']);
    let mut end = 0;
    let bytes = trimmed.as_bytes();
    if end < bytes.len() && bytes[end] == b'-' {
        end += 1;
    }
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    trimmed[..end].parse().unwrap_or(0.0)
}

/// Resolves a pattern to one installed version, `N/A` with status 3 when
/// nothing matches.
///
/// nvm: nvm_version
pub fn version(nvm: &mut Nvm, pattern: &str) -> (String, i32) {
    let mut pattern = if pattern.is_empty() {
        "current"
    } else {
        pattern
    }
    .to_string();
    if pattern == "current" {
        return (ls_current(nvm), 0);
    }
    if pattern == NODE_PREFIX || pattern == "node-" {
        pattern = "stable".to_string();
    }
    let (listing, _) = ls(nvm, &pattern);
    let mut last = listing.rsplit('\n').next().unwrap_or("").to_string();
    if last.starts_with("system") && last[6..].starts_with([' ', '\t']) {
        last = "system".to_string();
    }
    if last.is_empty() || last == "N/A" {
        return ("N/A".to_string(), 3);
    }
    (last, 0)
}

/// nvm: nvm_match_version
pub fn match_version(nvm: &mut Nvm, provided: &str) -> (String, i32) {
    match provided {
        IOJS_PREFIX | "io.js" => version(nvm, IOJS_PREFIX),
        "system" => ("system".to_string(), 0),
        _ => version(nvm, provided),
    }
}

/// Fails with nvm's "is not yet installed" message unless the version is
/// installed. Returns nvm's status.
///
/// nvm: nvm_ensure_version_installed
pub fn ensure_version_installed(nvm: &mut Nvm, provided: &str, from_nvmrc: bool) -> i32 {
    use crate::text::messages;
    if provided == "system" {
        if has_system_iojs(nvm) || has_system_node(nvm) {
            return 0;
        }
        nvm.io.err(messages::NO_SYSTEM_VERSION);
        return 1;
    }
    let (local_version, status) = version(nvm, provided);
    if status == 0 && is_version_installed(nvm, &local_version) {
        return 0;
    }
    let (resolved, resolved_status) = alias::resolve_alias(nvm, provided);
    if resolved_status == 0 {
        nvm.io
            .err(&messages::not_installed_alias(provided, &resolved));
    } else {
        let prefixed = version::ensure_prefix(provided);
        let shown = if prefixed.is_empty() {
            provided
        } else {
            &prefixed
        };
        nvm.io.err(&messages::not_installed(shown));
    }
    nvm.io.err("");
    if provided == "lts" {
        nvm.io.err(messages::LTS_IS_NOT_AN_ALIAS);
    } else if !from_nvmrc {
        nvm.io.err(&messages::run_install(provided));
    } else {
        nvm.io.err(messages::RUN_INSTALL_NVMRC);
    }
    1
}
