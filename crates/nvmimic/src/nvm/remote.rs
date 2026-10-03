//! Remote versions: the mirror, downloading with curl or wget, and reading
//! the mirror's `index.tab`.

use super::alias;
use super::paths;
use super::posix_re;
use super::version::{self, IOJS_PREFIX, NODE_PREFIX};
use crate::context::Nvm;
use crate::text::messages;
use std::process::{Command, Stdio};

/// Where releases of one flavor come from, or nvm's error status.
///
/// nvm: nvm_get_mirror
pub fn get_mirror(nvm: &mut Nvm, flavor: &str, kind: &str) -> Result<String, i32> {
    let mirror = match (flavor, kind) {
        ("node", "std") => match nvm.vars.get("NVM_NODEJS_ORG_MIRROR") {
            "" => "https://nodejs.org/dist".to_string(),
            value => value.to_string(),
        },
        ("iojs", "std") => match nvm.vars.get("NVM_IOJS_ORG_MIRROR") {
            "" => "https://iojs.org/dist".to_string(),
            value => value.to_string(),
        },
        _ => {
            nvm.io.err(messages::UNKNOWN_RELEASE_TYPE);
            return Err(1);
        }
    };
    let rejected = mirror.contains(['`', '\\', '\'', '(', ' '])
        || !posix_re::ere(r"^https?://[]a-zA-Z0-9._~:/@%[-]+$")
            .is_some_and(|re| re.is_match(&mirror));
    if rejected {
        nvm.io.err(messages::MIRROR_URL_ONLY);
        return Err(2);
    }
    Ok(mirror)
}

/// nvm: nvm_sanitize_auth_header
pub fn sanitize_auth_header(header: &str) -> String {
    header
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || " :_.+/=~-".contains(*c))
        .collect()
}

/// Whether nvm would ask curl for compressed transfers: curl has libz and is
/// at least 7.21.0.
///
/// nvm: nvm_curl_use_compression nvm_curl_libz_support nvm_curl_version
fn curl_use_compression(nvm: &mut Nvm) -> bool {
    if let Some(cached) = nvm.cache.borrow().curl_compression {
        return cached;
    }
    let output = Command::new("curl")
        .arg("-V")
        .env_clear()
        .envs(nvm.vars.exported())
        .stderr(Stdio::null())
        .output();
    let text = output
        .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
        .unwrap_or_default();
    let libz = text
        .lines()
        .any(|line| line.starts_with("Features:") && line.contains("libz"));
    let curl_version = text
        .lines()
        .find_map(|line| {
            let mut fields = line.split_whitespace();
            (fields.next() == Some("curl")).then(|| fields.next().unwrap_or("").to_string())
        })
        .map(|v| v.split('-').next().unwrap_or("").to_string())
        .unwrap_or_default();
    let use_it = libz && version::gte(&curl_version, "7.21.0");
    nvm.cache.borrow_mut().curl_compression = Some(use_it);
    use_it
}

/// The program and arguments `nvm_download` would run, or `None` when
/// neither curl nor wget is installed.
///
/// nvm: nvm_download
pub fn download_command(nvm: &mut Nvm, args: &[&str]) -> Option<(String, Vec<String>)> {
    let path = nvm.vars.get("PATH").to_string();
    let header = nvm.vars.get("NVM_AUTH_HEADER").to_string();
    let (program, mut argv): (&str, Vec<String>) = if paths::on_path(&path, "curl") {
        let mut argv = vec!["-q".to_string(), "--fail".to_string()];
        if curl_use_compression(nvm) {
            argv.insert(0, "--compressed".to_string());
        }
        argv.extend(args.iter().map(|arg| arg.to_string()));
        ("curl", argv)
    } else if paths::on_path(&path, "wget") {
        let mut argv = Vec::new();
        let mut skip = false;
        for arg in args {
            if skip {
                skip = false;
                continue;
            }
            match *arg {
                "--progress-bar" => argv.push("--progress=bar".to_string()),
                "--compressed" | "--fail" | "-L" => {}
                "-I" => argv.push("--server-response".to_string()),
                "-s" => argv.push("-q".to_string()),
                "-sS" => argv.push("-nv".to_string()),
                "-o" => argv.push("-O".to_string()),
                "-C" => {
                    skip = true;
                    argv.push("-c".to_string());
                }
                other => argv.push(other.to_string()),
            }
        }
        ("wget", argv)
    } else {
        return None;
    };
    if !header.is_empty() {
        argv.push("--header".to_string());
        argv.push(format!("Authorization: {}", sanitize_auth_header(&header)));
    }
    Some((program.to_string(), argv))
}

/// Runs a download and returns what it wrote to stdout. Its stderr goes to
/// nvm's stderr.
pub fn download_capture(nvm: &mut Nvm, args: &[&str]) -> Vec<u8> {
    let Some((program, argv)) = download_command(nvm, args) else {
        return Vec::new();
    };
    let mut command = Command::new(&program);
    command
        .args(&argv)
        .env_clear()
        .envs(nvm.vars.exported())
        .stdin(Stdio::null());
    if !nvm.vars.get("PWD").is_empty() {
        command.current_dir(nvm.vars.get("PWD"));
    }
    match command.output() {
        Ok(output) => {
            nvm.io.err_raw(&String::from_utf8_lossy(&output.stderr));
            output.stdout
        }
        Err(_) => Vec::new(),
    }
}

/// `index.tab` for a mirror, without its header line and with each line
/// prefixed. Downloaded once per call of nvmimic, which nvm does every time
/// it needs it.
fn index_lines(nvm: &mut Nvm, mirror: &str, prefix: &str) -> String {
    let url = format!("{mirror}/index.tab");
    let cached = nvm.cache.borrow().index_tab.get(&url).cloned();
    let body = match cached {
        Some(body) => body,
        None => {
            let bytes = download_capture(nvm, &["-L", "-s", &url, "-o", "-"]);
            let body = String::from_utf8_lossy(&bytes).into_owned();
            nvm.cache.borrow_mut().index_tab.insert(url, body.clone());
            body
        }
    };
    // `sed "1d; s/^/${PREFIX}/"`, then `$( ... )` drops trailing newlines.
    let mut lines: Vec<&str> = body.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let joined = lines
        .iter()
        .skip(1)
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n");
    joined.trim_end_matches('\n').to_string()
}

/// awk's default field splitting: runs of blanks, ignoring leading ones.
fn fields(line: &str) -> Vec<&str> {
    line.split([' ', '\t'])
        .filter(|field| !field.is_empty())
        .collect()
}

/// Lists one flavor's remote versions matching a pattern, writing the
/// `lts/*` and `lts/<codename>` aliases from the index on the way.
///
/// nvm: nvm_ls_remote_index_tab
pub fn ls_remote_index_tab(
    nvm: &mut Nvm,
    flavor: &str,
    kind: &str,
    pattern: Option<&str>,
) -> (String, i32) {
    let lts = nvm.vars.get("NVM_LTS").to_string();
    let Some(pattern) = pattern else {
        nvm.io.err(messages::NOT_ENOUGH_ARGUMENTS);
        return (String::new(), 5);
    };
    let Ok(mirror) = get_mirror(nvm, flavor, kind) else {
        return (String::new(), 3);
    };
    let prefix = match (flavor, kind) {
        ("iojs", "std") => format!("{IOJS_PREFIX}-"),
        ("node", "std") => String::new(),
        ("iojs", _) => {
            nvm.io.err(messages::UNKNOWN_IOJS_RELEASE);
            return (String::new(), 4);
        }
        _ => {
            nvm.io.err(messages::UNKNOWN_NODE_RELEASE);
            return (String::new(), 4);
        }
    };

    let mut pattern = pattern.to_string();
    if pattern.ends_with('.') {
        pattern.pop();
    }
    let pattern: Option<String> = if !pattern.is_empty() && pattern != "*" {
        Some(if flavor == "iojs" {
            version::ensure_prefix(version::strip_iojs_prefix(&pattern))
        } else {
            version::ensure_prefix(&pattern)
        })
    } else {
        None
    };

    let list = index_lines(nvm, &mirror, &prefix);
    let alias_dir = nvm.backend.alias_dir(&nvm.vars);
    nvm.backend.make_dirs(&format!("{alias_dir}/lts"));
    for (name, target) in lts_aliases(&list) {
        nvm.quiet(|nvm| alias::make_alias(nvm, &name, &target));
    }

    let mut lts = lts;
    if !lts.is_empty() {
        // nvm returns the status of `! LTS="$(...)"`, which is always 0.
        match alias::normalize_lts(nvm, &format!("lts/{lts}")) {
            Ok(normalized) => {
                lts = normalized
                    .strip_prefix("lts/")
                    .unwrap_or(&normalized)
                    .to_string()
            }
            Err(_) => return (String::new(), 0),
        }
    }

    let mut rows: Vec<String> = Vec::new();
    let mut prev = String::new();
    for line in format!("{list}\n").split('\n') {
        let f = fields(line);
        let first = f.first().copied().unwrap_or("");
        let codename = f.get(9).copied().unwrap_or("");
        if !super::awk::Field::Present(first).truthy() {
            continue;
        }
        let no_lts = codename.is_empty() || codename == "-";
        if !lts.is_empty() && no_lts {
            continue;
        }
        if !lts.is_empty()
            && lts != "*"
            && !posix_re::ere(&lts.to_lowercase())
                .is_some_and(|re| re.is_match(&codename.to_lowercase()))
        {
            continue;
        }
        if !no_lts {
            if super::awk::Field::Present(codename).truthy() && codename != prev {
                rows.push(format!("{first} {codename} *"));
            } else {
                rows.push(format!("{first} {codename}"));
            }
        } else {
            rows.push(first.to_string());
        }
        prev = codename.to_string();
    }

    // `grep -w "${PATTERN:-.*}"`: the pattern must match as a whole word.
    let word = pattern.as_deref().unwrap_or(".*");
    let matcher = posix_re::bre(word).and_then(|re| {
        regex::Regex::new(&format!(
            "(^|[^[:alnum:]_])(?:{})([^[:alnum:]_]|$)",
            re.as_str()
        ))
        .ok()
    });
    rows.retain(|row| matcher.as_ref().is_some_and(|re| re.is_match(row)));

    if flavor == "node" {
        super::local::sort_versions(&mut rows);
    } else {
        rows.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    }

    if rows.is_empty() {
        return ("N/A".to_string(), 3);
    }
    (rows.join("\n"), 0)
}

/// The LTS aliases an index implies: `lts/*` for the newest line, and for
/// each line its newest version, in nvm's order.
fn lts_aliases(list: &str) -> Vec<(String, String)> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    let mut current: Option<(String, String)> = None;
    for line in format!("{list}\n").split('\n') {
        let f = fields(line);
        let codename = f.get(9).copied().unwrap_or("");
        if codename.is_empty() || codename == "-" {
            continue;
        }
        let lower = codename.to_lowercase();
        let valid = posix_re::ere("^[a-z0-9][a-z0-9._-]*$").is_some_and(|re| re.is_match(&lower));
        if !valid {
            continue;
        }
        if super::awk::Field::Present(codename).truthy() && seen.insert(lower.clone()) {
            let name = format!("lts/{lower}");
            match current.take() {
                Some(previous) => out.push(previous),
                None => out.push(("lts/*".to_string(), name.clone())),
            }
            current = Some((name, f.first().copied().unwrap_or("").to_string()));
        }
    }
    if let Some(last) = current {
        out.push(last);
    }
    out
}

/// nvm: nvm_ls_remote
pub fn ls_remote(nvm: &mut Nvm, pattern: &str) -> (String, i32) {
    let pattern = if version::is_implicit_alias(pattern) {
        let implicit = print_implicit_alias_remote(nvm, pattern).0;
        if implicit.is_empty() || implicit == "N/A" {
            return ("N/A".to_string(), 3);
        }
        let listing = ls_remote(nvm, &implicit).0;
        listing
            .rsplit('\n')
            .next()
            .and_then(|line| fields(line).first().map(|f| f.to_string()))
            .unwrap_or_default()
    } else if !pattern.is_empty() {
        version::ensure_prefix(pattern)
    } else {
        ".*".to_string()
    };
    ls_remote_index_tab(nvm, "node", "std", Some(&pattern))
}

/// nvm: nvm_ls_remote_iojs
pub fn ls_remote_iojs(nvm: &mut Nvm, pattern: &str) -> (String, i32) {
    ls_remote_index_tab(nvm, "iojs", "std", Some(pattern))
}

/// The remote value of `stable`, `unstable`, `node` or `iojs`.
///
/// nvm: nvm_print_implicit_alias
pub fn print_implicit_alias_remote(nvm: &mut Nvm, implicit: &str) -> (String, i32) {
    if !version::is_implicit_alias(implicit) {
        nvm.io.err(&messages::only_implicit_aliases());
        return (String::new(), 2);
    }
    match implicit {
        IOJS_PREFIX => {
            let (listing, status) = ls_remote_iojs(nvm, "");
            let mut found = listing.clone();
            if status == 0 {
                found = minors_of(
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
            let listing = ls_remote(nvm, "").0;
            let mut stable = String::new();
            let mut unstable = String::new();
            for minor in minors_of(listing.lines()) {
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
fn minors_of<'s>(lines: impl Iterator<Item = &'s str>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in lines.filter(|line| line.starts_with('v')) {
        let minor = line[1..].split('.').take(2).collect::<Vec<_>>().join(".");
        if out.last() != Some(&minor) {
            out.push(minor);
        }
    }
    out
}

/// Node and io.js remote versions together, io.js between Node 0.x and 4.
///
/// nvm: nvm_remote_versions
pub fn remote_versions(nvm: &mut Nvm, pattern: &str) -> (String, i32) {
    let lts = !nvm.vars.get("NVM_LTS").is_empty();
    let mut flavor = if lts { Some(NODE_PREFIX) } else { None };
    let mut pattern = Some(pattern.to_string());
    match pattern.as_deref() {
        Some(IOJS_PREFIX) | Some("io.js") => {
            flavor = Some(IOJS_PREFIX);
            pattern = None;
        }
        Some(NODE_PREFIX) => {
            flavor = Some(NODE_PREFIX);
            pattern = None;
        }
        _ => {}
    }
    let pattern = pattern.unwrap_or_default();
    if version::is_implicit_alias(&pattern) {
        nvm.io.err(messages::IMPLICIT_NOT_SUPPORTED);
        return (String::new(), 1);
    }

    let mut node_status = 0;
    let (mut pre, mut post) = (String::new(), String::new());
    if flavor.is_none() || flavor == Some(NODE_PREFIX) {
        let (listing, status) = ls_remote(nvm, &pattern);
        node_status = status;
        let output = format!("{listing} ");
        match output.find("v4.0.0") {
            Some(at) => {
                pre = output[..at].to_string();
                post = output[at..].to_string();
            }
            None => pre = output,
        }
    }

    let mut iojs_status = 0;
    let mut iojs = String::new();
    if !lts && (flavor.is_none() || flavor == Some(IOJS_PREFIX)) {
        let (listing, status) = ls_remote_iojs(nvm, &pattern);
        iojs = listing;
        iojs_status = status;
    }

    let combined = format!("{pre}\n{iojs}\n{post}");
    let versions: Vec<&str> = combined
        .split('\n')
        .filter(|line| !line.contains("N/A"))
        .filter(|line| !line.chars().all(|c| c == ' '))
        .collect();
    if versions.is_empty() {
        return ("N/A".to_string(), 3);
    }
    let text = versions
        .iter()
        .map(|line| line.trim_end_matches(' '))
        .collect::<Vec<_>>()
        .join("\n");
    let status = if node_status != 0 {
        node_status
    } else {
        iojs_status
    };
    (text, status)
}

/// The newest remote version matching a pattern, `N/A` with status 3 when none.
///
/// nvm: nvm_remote_version
pub fn remote_version(nvm: &mut Nvm, pattern: &str) -> (String, i32) {
    let mut version = if version::is_implicit_alias(pattern) {
        if pattern == IOJS_PREFIX {
            let listing = ls_remote_iojs(nvm, "").0;
            listing.rsplit('\n').next().unwrap_or("").to_string()
        } else {
            ls_remote(nvm, pattern).0
        }
    } else {
        let listing = remote_versions(nvm, pattern).0;
        listing.rsplit('\n').next().unwrap_or("").to_string()
    };

    if !pattern.is_empty() && version != "N/A" && !version::is_implicit_alias(pattern) {
        let number = fields(&version).first().copied().unwrap_or("").to_string();
        let found = posix_re::bre(pattern).is_some_and(|re| re.is_match(&number));
        if !found {
            version = "N/A".to_string();
        }
    }

    let shown = if !nvm.vars.get("NVM_VERSION_ONLY").is_empty() {
        fields(&version).first().copied().unwrap_or("").to_string()
    } else {
        version.clone()
    };
    if version == "N/A" {
        (shown, 3)
    } else {
        (shown, 0)
    }
}
