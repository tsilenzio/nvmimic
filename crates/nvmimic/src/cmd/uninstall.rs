//! `nvm uninstall`: removes one installed version, its unpacked downloads,
//! and the aliases that point at it.

use super::cache::cache_dir;
use crate::backend;
use crate::context::Nvm;
use crate::nvm::{artifact, local, paths, posix_re, tools, version};
use crate::text::install as text;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// nvm: nvm:uninstall
pub fn run(nvm: &mut Nvm, args: &[String]) -> i32 {
    let [pattern] = args else {
        for line in text::UNINSTALL_USAGE {
            nvm.io.err(line);
        }
        return 127;
    };

    // `local VERSION` in nvm() has no value, so for a pattern of `--`,
    // which assigns nothing, dash is left with the caller's `VERSION`.
    let saved = nvm.local("VERSION");
    let mut version = nvm.vars.get("VERSION").to_string();
    nvm.restore("VERSION", saved);
    match pattern.as_str() {
        "--" => {}
        "--lts" | "lts/*" => version = local::match_version(nvm, "lts/*").0,
        other => {
            version = if other.starts_with("lts/") {
                local::match_version(nvm, other).0
            } else if let Some(name) = other.strip_prefix("--lts=") {
                local::match_version(nvm, &format!("lts/{name}")).0
            } else {
                local::version(nvm, other).0
            }
        }
    }

    let is_iojs = version::is_iojs(&version);
    if version == local::ls_current(nvm) {
        nvm.io.err(&match is_iojs {
            true => text::cannot_uninstall_active_iojs(&version, pattern),
            false => text::cannot_uninstall_active_node(&version, pattern),
        });
        return 1;
    }

    if !local::is_version_installed(nvm, &version) {
        if version != "N/A" && &version != pattern {
            nvm.io.err(&text::inferred_not_installed(&version, pattern));
        } else {
            nvm.io.err(&text::version_not_installed(pattern));
        }
        return 0;
    }

    // nvm hands nvm_get_download_slug four arguments where it takes three,
    // so the version it sees is the literal `std`. The names that come out
    // match no download, and the unpacked files of this version stay in the
    // cache. Kept as it is.
    let flavor = if is_iojs { "iojs" } else { "node" };
    let slug_binary = artifact::get_download_slug(nvm, flavor, "binary", "std").unwrap_or_default();
    let slug_source = artifact::get_download_slug(nvm, flavor, "source", "std").unwrap_or_default();

    let success = match is_iojs {
        true => text::uninstalled_iojs(version::strip_iojs_prefix(&version)),
        false => text::uninstalled_node(&version),
    };

    let version_path = paths::version_path(nvm, &version).unwrap_or_else(|message| {
        nvm.io.err(&message);
        String::new()
    });
    let (allowed, last) = check_file_permissions(nvm, &version_path);
    // The loop variable `FILE` is not declared `local`, so it stays set.
    nvm.vars.assign("FILE", last);
    if allowed != 0 {
        nvm.io.err(text::UNINSTALL_PERMISSIONS);
        nvm.io.err(text::UNINSTALL_PERMISSIONS_HINT);
        nvm.io.err("");
        let user = tools::run(nvm, "whoami", &[])
            .map(|ran| ran.text())
            .unwrap_or_default();
        let shown = paths::sanitize(&version_path, nvm.vars.get("NVM_DIR"), nvm.vars.get("HOME"));
        nvm.io.err(&text::chown_hint(&user, &shown));
        nvm.io.err(&text::chmod_hint(&shown));
        return 1;
    }

    let cache = cache_dir(nvm);
    let cache_removed = tools::rm_rf_quiet(&[
        &format!("{cache}/bin/{slug_binary}/files"),
        &format!("{cache}/src/{slug_source}/files"),
    ]);
    if cache_removed && !version_path.is_empty() {
        let _ = nvm.backend.remove(&nvm.vars, &version);
    }
    nvm.io.echo(&success);

    // Aliases that name the version go with it. nvm finds them with
    // `grep -l`, so the version is read as a pattern.
    let mentions = posix_re::bre(&version);
    let alias_dir = nvm.backend.alias_dir(&nvm.vars);
    let mut status = 0;
    for entry in backend::dir_entries(&alias_dir) {
        let file = format!("{alias_dir}/{}", entry.name);
        if !Path::new(&file).is_file() {
            continue;
        }
        let content = fs::read(&file).unwrap_or_default();
        let found = String::from_utf8_lossy(&content)
            .lines()
            .any(|line| mentions.as_ref().is_some_and(|re| re.is_match(line)));
        if found {
            // Another loop variable nvm leaves behind.
            nvm.vars.assign("ALIAS", file);
            status = super::nvm(nvm, &["unalias".to_string(), entry.name]);
        }
    }
    status
}

/// What `"${dir}"/* "${dir}"/.[!.]* "${dir}"/..?*` expands to: plain names,
/// then dot files, then names starting with two dots, each group sorted. A
/// pattern that matches nothing stays as written.
fn all_entries(dir: &str) -> Vec<String> {
    let mut plain = Vec::new();
    let mut dotted = Vec::new();
    let mut double_dotted = Vec::new();
    for name in fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
    {
        if !name.starts_with('.') {
            plain.push(name);
        } else if name.starts_with("..") {
            double_dotted.push(name);
        } else {
            dotted.push(name);
        }
    }
    let mut all = Vec::new();
    for (mut names, pattern) in [(plain, "*"), (dotted, ".[!.]*"), (double_dotted, "..?*")] {
        names.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        if names.is_empty() {
            names.push(pattern.to_string());
        }
        all.extend(names.into_iter().map(|name| format!("{dir}/{name}")));
    }
    all
}

/// Checks that everything in a version's directory can be removed by this
/// user: each file is writable or owned by them. Returns nvm's status and
/// the last path its loop looked at.
///
/// nvm: nvm_check_file_permissions
pub fn check_file_permissions(nvm: &mut Nvm, dir: &str) -> (i32, String) {
    let mut last = String::new();
    for file in all_entries(dir) {
        last = file.clone();
        let path = Path::new(&file);
        if path.is_dir() {
            if !nvm.vars.get("NVM_DEBUG").is_empty() {
                nvm.io.err(&file);
            }
            let is_link =
                fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink());
            if !is_link {
                let (status, inner) = check_file_permissions(nvm, &file);
                last = inner;
                if status != 0 {
                    return (2, last);
                }
                // Back from the directory, nvm's loop variable holds what
                // the inner loop left in it.
            }
        } else if path.exists() && !writable(&file) && !owned(&file) {
            let shown = paths::sanitize(&file, nvm.vars.get("NVM_DIR"), nvm.vars.get("HOME"));
            nvm.io.err(&text::not_writable(&shown));
            return (1, last);
        }
    }
    (0, last)
}

/// `[ -w path ]`.
fn writable(path: &str) -> bool {
    use std::os::unix::ffi::OsStrExt;
    let Ok(c_path) = std::ffi::CString::new(Path::new(path).as_os_str().as_bytes()) else {
        return false;
    };
    // SAFETY: `c_path` is a valid NUL-terminated string for the whole call.
    unsafe { libc::access(c_path.as_ptr(), libc::W_OK) == 0 }
}

/// `find path -prune -user "$(id -u)"`: whether the file itself, not what a
/// link points to, belongs to this user.
fn owned(path: &str) -> bool {
    // SAFETY: geteuid has no arguments and cannot fail.
    let me = unsafe { libc::geteuid() };
    fs::symlink_metadata(path).is_ok_and(|meta| meta.uid() == me)
}
