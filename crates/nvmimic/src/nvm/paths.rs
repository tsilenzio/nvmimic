//! Paths: where versions live, walking up from the working directory, and
//! editing `PATH`-style lists the way nvm does.

use super::posix_re::{self, sed_first};
use crate::backend;
use crate::context::Nvm;
use std::path::Path;

/// Pure string containment: is `node_path` inside `tree`?
///
/// nvm: nvm_tree_contains_path
pub fn tree_contains_path(tree: &str, node_path: &str) -> Option<bool> {
    if tree.is_empty() || node_path.is_empty() {
        return None;
    }
    let mut clean = tree.trim_end_matches('/');
    if clean.is_empty() {
        clean = "/";
    }
    let mut dir = node_path.to_string();
    while !dir.is_empty() && dir != "." && dir != "/" && dir != clean {
        match dir.rfind('/') {
            Some(at) => {
                dir.truncate(at);
                if dir.is_empty() {
                    dir = "/".to_string();
                }
            }
            None => break,
        }
    }
    Some(dir == clean)
}

/// The nearest directory at or above `$PWD` holding a file named `name`, or
/// an empty string. nvm walks the logical `$PWD` with `${path%/*}`.
///
/// nvm: nvm_find_up
pub fn find_up(pwd: &str, name: &str) -> String {
    let mut dir = pwd.to_string();
    while !dir.is_empty() && dir != "." && !Path::new(&format!("{dir}/{name}")).is_file() {
        match dir.rfind('/') {
            Some(at) => dir.truncate(at),
            None => break,
        }
    }
    dir
}

/// nvm: nvm_find_nvmrc
pub fn find_nvmrc(pwd: &str) -> Option<String> {
    let dir = find_up(pwd, ".nvmrc");
    let path = format!("{dir}/.nvmrc");
    Path::new(&path).exists().then_some(path)
}

/// nvm: nvm_find_project_dir
pub fn find_project_dir(pwd: &str) -> String {
    let mut dir = pwd.to_string();
    while !dir.is_empty()
        && dir != "."
        && !Path::new(&format!("{dir}/package.json")).is_file()
        && !Path::new(&format!("{dir}/node_modules")).is_dir()
    {
        match dir.rfind('/') {
            Some(at) => dir.truncate(at),
            None => break,
        }
    }
    dir
}

/// Where a version is installed, or the error nvm prints and status 3.
///
/// nvm: nvm_version_path
pub fn version_path(nvm: &Nvm, version: &str) -> Result<String, String> {
    if format!("/{version}/").contains("/../") {
        return Err(format!("invalid version: {version}"));
    }
    if version.is_empty() {
        return Err("version is required".to_string());
    }
    Ok(nvm.backend.version_path(&nvm.vars, version))
}

/// Rewrites `$NVM_DIR` and `$HOME` in a path to the literal text `${NVM_DIR}`
/// and `${HOME}`, for messages.
///
/// nvm: nvm_sanitize_path
pub fn sanitize(path: &str, nvm_dir: &str, home: &str) -> String {
    let mut out = path.to_string();
    if out != nvm_dir {
        out = sed_global_literalish(&out, nvm_dir, "${NVM_DIR}");
    }
    if out != home {
        out = sed_global_literalish(&out, home, "${HOME}");
    }
    out
}

/// `sed -e "s#${needle}#${replacement}#g"`, where the needle is read as a
/// basic regular expression the way sed reads it.
fn sed_global_literalish(text: &str, needle: &str, replacement: &str) -> String {
    match posix_re::bre(needle) {
        Some(re) if !needle.is_empty() => re
            .replace_all(text, |caps: &regex::Captures| {
                posix_re::sed_replacement(replacement, caps)
            })
            .into_owned(),
        _ => text.to_string(),
    }
}

/// Removes entries under `$NVM_DIR` ending in `suffix` from a colon list,
/// keeping a trailing colon if there was one.
///
/// nvm: nvm_strip_path
pub fn strip_path(list: &str, nvm_dir: &str, suffix: &str) -> String {
    let re = posix_re::ere(&format!("^(/versions/[^/]*)?/[^/]*{suffix}.*$"));
    let body = list.strip_suffix(':').unwrap_or(list);
    let mut kept: Vec<&str> = Vec::new();
    if !list.is_empty() {
        for entry in body.split(':') {
            if let Some(rest) = entry.strip_prefix(nvm_dir)
                && re.as_ref().is_some_and(|re| re.is_match(rest))
            {
                continue;
            }
            kept.push(entry);
        }
    }
    let mut result = kept.join(":");
    if list.ends_with(':') {
        result.push(':');
    }
    result
}

/// Puts `dir + suffix` into a colon list in place of the nvm entry already
/// there, or in front when there is none.
///
/// nvm: nvm_change_path
pub fn change_path(list: &str, suffix: &str, dir: &str, nvm_dir: &str) -> String {
    if list.is_empty() {
        return format!("{dir}{suffix}");
    }
    let flat = format!("{nvm_dir}/[^/]*{suffix}");
    let versioned = format!("{nvm_dir}/versions/[^/]*/[^/]*{suffix}");
    let grep = |pattern: &str| posix_re::bre(pattern).is_some_and(|re| re.is_match(list));
    let grep_e = |pattern: &str| posix_re::ere(pattern).is_some_and(|re| re.is_match(list));
    if !grep(&flat) && !grep(&versioned) {
        return format!("{dir}{suffix}:{list}");
    }
    if grep_e(&format!("(^|:)(/usr(/local)?)?{suffix}:.*{flat}"))
        || grep_e(&format!("(^|:)(/usr(/local)?)?{suffix}:.*{versioned}"))
    {
        return format!("{dir}{suffix}:{list}");
    }
    let replacement = format!("{dir}{suffix}");
    let mut line = list.to_string();
    for pattern in [format!("{flat}[^:]*"), format!("{versioned}[^:]*")] {
        if let Some(re) = posix_re::bre(&pattern) {
            line = sed_first(&line, &re, &replacement);
        }
    }
    line
}

/// What `command which <name>` prints: the first executable regular file
/// named `name` in `PATH`. An empty `PATH` entry means the current directory.
pub fn which(path: &str, name: &str) -> Option<String> {
    for dir in path.split(':') {
        let dir = if dir.is_empty() { "." } else { dir };
        let candidate = format!("{dir}/{name}");
        let file = Path::new(&candidate);
        if file.is_file() && backend::is_executable(file) {
            return Some(candidate);
        }
    }
    None
}

/// Whether `command -v <name>` finds an executable in `PATH`.
pub fn on_path(path: &str, name: &str) -> bool {
    which(path, name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn containment() {
        assert_eq!(
            tree_contains_path("/a/.nvm", "/a/.nvm/versions/node/v1/bin/node"),
            Some(true)
        );
        assert_eq!(tree_contains_path("/a/.nvm/", "/a/.nvm/x"), Some(true));
        assert_eq!(tree_contains_path("/a/.nvm", "/a/.nvmx/node"), Some(false));
        assert_eq!(tree_contains_path("", "/x"), None);
    }

    #[test]
    fn strip_removes_nvm_entries() {
        let dir = "/h/.nvm";
        assert_eq!(
            strip_path("/h/.nvm/versions/node/v22.1.0/bin:/usr/bin", dir, "/bin"),
            "/usr/bin"
        );
        assert_eq!(strip_path("/usr/bin:", dir, "/bin"), "/usr/bin:");
        assert_eq!(strip_path("", dir, "/bin"), "");
        assert_eq!(strip_path("/h/.nvm/v0.10.1/bin:/bin", dir, "/bin"), "/bin");
    }

    #[test]
    fn change_replaces_in_place_or_prepends() {
        let dir = "/h/.nvm";
        let v20 = "/h/.nvm/versions/node/v20.0.0";
        assert_eq!(
            change_path("/usr/bin", "/bin", v20, dir),
            format!("{v20}/bin:/usr/bin")
        );
        assert_eq!(
            change_path(
                "/x:/h/.nvm/versions/node/v22.1.0/bin:/usr/bin",
                "/bin",
                v20,
                dir
            ),
            format!("/x:{v20}/bin:/usr/bin")
        );
        assert_eq!(
            change_path(
                "/usr/bin:/h/.nvm/versions/node/v22.1.0/bin",
                "/bin",
                v20,
                dir
            ),
            format!("{v20}/bin:/usr/bin:/h/.nvm/versions/node/v22.1.0/bin")
        );
        assert_eq!(
            change_path("", "/share/man", v20, dir),
            format!("{v20}/share/man")
        );
    }
}
