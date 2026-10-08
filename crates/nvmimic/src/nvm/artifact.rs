//! Getting a Node release onto disk: naming the archive, downloading and
//! checking it, unpacking it, and building from source when there is no
//! binary. curl or wget, the checksum program, tar and make all run as real
//! programs here, the same ones nvm runs.

use super::checksum;
use super::complaint::{self, Site, sites};
use super::local;
use super::paths;
use super::platform;
use super::posix_re;
use super::remote;
use super::tools;
use super::version;
use crate::cmd::cache::cache_dir;
use crate::context::Nvm;
use crate::text::install as text;
use std::fs;
use std::path::Path;

/// `[ -r path ]`.
fn readable(path: &str) -> bool {
    use std::os::unix::ffi::OsStrExt;
    let Ok(c_path) = std::ffi::CString::new(Path::new(path).as_os_str().as_bytes()) else {
        return false;
    };
    // SAFETY: `c_path` is a valid NUL-terminated string for the whole call.
    unsafe { libc::access(c_path.as_ptr(), libc::R_OK) == 0 }
}

/// What the `dirname` program prints for a path.
fn dirname(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    match trimmed.rfind('/') {
        Some(0) => "/".to_string(),
        Some(at) => trimmed[..at].trim_end_matches('/').to_string(),
        None if path.starts_with('/') => "/".to_string(),
        None => ".".to_string(),
    }
}

fn sanitized(nvm: &Nvm, path: &str) -> String {
    paths::sanitize(path, nvm.vars.get("NVM_DIR"), nvm.vars.get("HOME"))
}

/// Removes a download's work directory, as nvm's cleanup does, but only
/// when it lies inside `$NVM_DIR` or the backend's cache folder. Under
/// dash, nvm 0.40.8 can be left holding the caller's own `$TMPDIR` here and
/// removes that. nvmimic does not delete anything outside those two.
fn remove_work_dir(nvm: &mut Nvm, dir: &str) {
    let nvm_dir = nvm.vars.get("NVM_DIR").to_string();
    let cache = cache_dir(nvm);
    if paths::tree_contains_path(&nvm_dir, dir) == Some(true)
        || paths::tree_contains_path(&cache, dir) == Some(true)
    {
        tools::rm_rf(nvm, &[dir]);
    }
}

/// The name of a release's archive without its extension, such as
/// `node-v22.20.0-darwin-arm64`, or nvm's error status.
///
/// nvm: nvm_get_download_slug
pub fn get_download_slug(
    nvm: &mut Nvm,
    flavor: &str,
    kind: &str,
    version: &str,
) -> Result<String, i32> {
    if !matches!(flavor, "node" | "iojs") {
        nvm.io.err(text::SUPPORTED_FLAVORS);
        return Err(1);
    }
    if !matches!(kind, "binary" | "source") {
        nvm.io.err(text::SUPPORTED_KINDS);
        return Err(2);
    }
    let os = platform::get_os(nvm);
    let mut arch = platform::get_arch(nvm);
    if platform::is_merged_node_version(version) != 0
        && matches!(arch.as_str(), "armv6l" | "armv7l")
    {
        arch = "arm-pi".to_string();
    }
    // nodejs.org has no Apple Silicon builds before Node 16.
    if version::greater("16.0.0", version) && os == "darwin" && arch == "arm64" {
        arch = "x64".to_string();
    }
    Ok(match kind {
        "binary" => format!("{flavor}-{version}-{os}-{arch}"),
        _ => format!("{flavor}-{version}"),
    })
}

/// nvm: nvm_get_artifact_compression
pub fn get_artifact_compression(nvm: &mut Nvm, version: &str) -> &'static str {
    if platform::get_os(nvm) == "win" {
        "zip"
    } else if platform::supports_xz(nvm, version) == 0 {
        "tar.xz"
    } else {
        "tar.gz"
    }
}

/// Downloads one archive into the cache, or finds it there already, checks
/// it against the mirror's checksum, and prints its path. Returns nvm's
/// status.
///
/// nvm: nvm_download_artifact
pub fn download_artifact(
    nvm: &mut Nvm,
    flavor: &str,
    kind: &str,
    release_type: &str,
    version: &str,
) -> i32 {
    if !matches!(flavor, "node" | "iojs") {
        nvm.io.err(text::SUPPORTED_FLAVORS);
        return 1;
    }
    if !matches!(kind, "binary" | "source") {
        nvm.io.err(text::SUPPORTED_KINDS);
        return 1;
    }
    let Ok(mirror) = remote::get_mirror(nvm, flavor, release_type) else {
        return 2;
    };
    if version.is_empty() {
        nvm.io.err(text::VERSION_NUMBER_REQUIRED);
        return 3;
    }
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._+-".contains(c))
    {
        nvm.io.err(text::VERSION_DISALLOWED);
        return 3;
    }
    if kind == "binary" && platform::binary_available(version) != 0 {
        nvm.io.err(&text::no_binary(version));
        return 0;
    }

    let slug = get_download_slug(nvm, flavor, kind, version).unwrap_or_default();
    let compression = get_artifact_compression(nvm, version);
    let place = if kind == "binary" { "bin" } else { "src" };
    let tmpdir = format!("{}/{place}/{slug}", cache_dir(nvm));
    let tarball = format!("{tmpdir}/{slug}.{compression}");

    if nvm.vars.get("NVM_OFFLINE") == "1" {
        if readable(&tarball) {
            let shown = sanitized(nvm, &tarball);
            nvm.io.err(&text::offline_using(&shown));
            nvm.io.echo(&tarball);
            return 0;
        }
        nvm.io.err(&text::offline_missing(&slug));
        return 4;
    }

    let (listed, _) =
        checksum::get_checksum(nvm, flavor, release_type, version, &slug, compression);
    let expected = listed.trim_end_matches('\n');

    let files = format!("{tmpdir}/files");
    if !tools::mkdir_p(nvm, &files) {
        nvm.io.err(&text::creating_directory_failed(&files));
        return 3;
    }

    // Node 0.1.13 and older sit at the top of the mirror, with no directory.
    let url = if version::gte(version, "0.1.14") {
        format!("{mirror}/{version}/{slug}.{compression}")
    } else {
        format!("{mirror}/{slug}.{compression}")
    };

    if readable(&tarball) {
        let shown = sanitized(nvm, &tarball);
        nvm.io.err(&text::local_cache_found(&shown));
        if nvm.quiet(|nvm| checksum::compare_checksum(nvm, &tarball, expected)) == 0 {
            nvm.io.err(&text::using_existing_archive(&shown));
            nvm.io.echo(&tarball);
            return 0;
        }
        checksum::compare_checksum(nvm, &tarball, expected);
        nvm.io.err(text::CHECKSUM_CHECK_FAILED);
        nvm.io.err(text::REMOVING_BROKEN_CACHE);
        tools::rm_rf(nvm, &[&tarball]);
    }

    nvm.io.err(&text::downloading(&url));
    let progress = nvm.vars.get("PROGRESS_BAR").to_string();
    if remote::download(nvm, &["-L", "-C", "-", &progress, &url, "-o", &tarball]) != 0 {
        tools::rm_rf(nvm, &[&tarball, &tmpdir]);
        nvm.io.err(&text::download_failed(&url));
        return 4;
    }

    // Some mirrors answer a missing file with a page instead of an error.
    match fs::read(&tarball) {
        Ok(bytes) if bytes.windows(13).any(|window| window == b"404 Not Found") => {
            tools::rm_rf(nvm, &[&tarball, &tmpdir]);
            nvm.io.err(&text::http_404(&url));
            return 5;
        }
        Ok(_) => {}
        Err(error) => {
            let reason = crate::text::messages::strerror(&error);
            nvm.io.err(&format!("grep: {tarball}: {reason}"));
        }
    }

    if checksum::compare_checksum(nvm, &tarball, expected) != 0 {
        tools::rm_rf(nvm, &[&files]);
        return 6;
    }
    nvm.io.echo(&tarball);
    0
}

/// Unpacks an archive into a directory, without the archive's own top
/// directory. `args` are nvm's four: OS, version, archive, directory.
///
/// nvm: nvm_extract_tarball
pub fn extract_tarball(nvm: &mut Nvm, args: &[&str]) -> i32 {
    let &[os, version, tarball, tmpdir] = args else {
        nvm.io.err(text::EXTRACT_ARGUMENTS);
        return 5;
    };
    let xz = platform::supports_xz(nvm, version) == 0;
    let tar = if os == "aix" { "gtar" } else { "tar" };

    let status = if os == "openbsd" {
        // OpenBSD's tar has no `--strip-components` and cannot read xz.
        let strip = ["-C", tmpdir, "-s", r"/[^\/]*\///"];
        if xz {
            let unpacked = tools::run(nvm, "xzcat", &[tarball])
                .map(|ran| ran.stdout)
                .unwrap_or_default();
            match tools::command(nvm, tar) {
                Some(mut command) => {
                    command.args(["-xf", "-"]).args(strip);
                    tools::run_fed(nvm, command, &unpacked)
                }
                None => 127,
            }
        } else {
            run_program(nvm, tar, &[&["-xzf", tarball], &strip[..]].concat())
        }
    } else {
        let flag = if xz { "-xJf" } else { "-xzf" };
        run_program(
            nvm,
            tar,
            &[
                flag,
                tarball,
                "-C",
                tmpdir,
                "--strip-components",
                "1",
                "--no-same-owner",
            ],
        )
    };
    (status != 0) as i32
}

/// Runs a program from `PATH` in the foreground and returns its status,
/// 127 when it is not there.
fn run_program(nvm: &mut Nvm, program: &str, args: &[&str]) -> i32 {
    match tools::command(nvm, program) {
        Some(mut command) => {
            command.args(args);
            tools::run_through(nvm, command)
        }
        None => 127,
    }
}

/// Unpacks a downloaded binary release and moves it to where the version
/// lives. `args` are nvm's five: OS, version as given, version, archive,
/// work directory.
///
/// nvm: nvm_install_binary_extract
pub fn install_binary_extract(nvm: &mut Nvm, args: &[&str]) -> i32 {
    let &[os, prefixed_version, version, tarball, tmpdir] = args else {
        nvm.io.err(text::BINARY_EXTRACT_ARGUMENTS);
        return 1;
    };
    if tmpdir.is_empty() || !tools::mkdir_p(nvm, tmpdir) {
        return 1;
    }
    let mut version_path = match paths::version_path(nvm, prefixed_version) {
        Ok(path) => path,
        Err(message) => {
            nvm.io.err(&message);
            return 1;
        }
    };

    // Git Bash and Cygwin: a zip with `node.exe` at its top.
    if os == "win" {
        version_path.push_str("/bin");
        if run_program(nvm, "unzip", &["-q", tarball, "-d", tmpdir]) != 0 {
            return 1;
        }
        tools::rm_rf(nvm, &[&version_path]);
        if !tools::mkdir_p(nvm, &version_path) {
            return 1;
        }
        let mut moved: Vec<String> = Vec::new();
        for top in visible_entries(tmpdir) {
            moved.extend(visible_entries(&top));
        }
        if !move_into(
            nvm,
            &moved,
            &format!("{tmpdir}/*/*"),
            &format!("{version_path}/"),
        ) {
            return 1;
        }
        for name in ["node.exe", "npm"] {
            if run_program(nvm, "chmod", &["+x", &format!("{version_path}/{name}")]) != 0 {
                return 1;
            }
        }
        nvm.quiet_err(|nvm| run_program(nvm, "chmod", &["+x", &format!("{version_path}/npx")]));
        tools::rm_rf(nvm, &[tmpdir]);
        return 0;
    }

    if extract_tarball(nvm, &[os, version, tarball, tmpdir]) != 0 {
        return 1;
    }

    // One rename puts the whole tree in place, so a half-installed version
    // is never seen. Whatever was there before is removed first.
    if !tools::rm_rf(nvm, &[&version_path]) || !tools::mkdir_p(nvm, &dirname(&version_path)) {
        return 1;
    }
    if fs::rename(tmpdir, &version_path).is_ok() {
        return 0;
    }

    // The cache and the versions directory are on different filesystems.
    tools::rm_rf(nvm, &[&version_path]);
    if !tools::mkdir_p(nvm, &version_path) {
        return 1;
    }
    if !move_into(
        nvm,
        &visible_entries(tmpdir),
        &format!("{tmpdir}/*"),
        &version_path,
    ) {
        return 1;
    }
    tools::rm_rf(nvm, &[tmpdir]);
    0
}

/// What `"${dir}"/*` expands to: the entries whose names do not start with
/// a dot, sorted.
fn visible_entries(dir: &str) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .collect();
    names.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    names
        .into_iter()
        .map(|name| format!("{dir}/{name}"))
        .collect()
}

/// `command mv <sources> <target>`, where the sources came from a glob.
/// A glob that matched nothing is passed along as written, as a shell does.
fn move_into(nvm: &mut Nvm, sources: &[String], pattern: &str, target: &str) -> bool {
    let mut args: Vec<&str> = sources.iter().map(String::as_str).collect();
    if args.is_empty() {
        args.push(pattern);
    }
    args.push(target);
    run_program(nvm, "mv", &args) == 0
}

/// Downloads a binary release and installs it. Returns nvm's status: 0 when
/// installed, 1 when the caller should try building from source, 2 when the
/// download failed and building from source was ruled out.
///
/// nvm: nvm_install_binary
pub fn install_binary(
    nvm: &mut Nvm,
    flavor: &str,
    release_type: &str,
    prefixed_version: &str,
    nosource: &str,
) -> i32 {
    if !matches!(flavor, "node" | "iojs") {
        nvm.io.err(text::SUPPORTED_FLAVORS);
        return 4;
    }
    if prefixed_version.is_empty() {
        nvm.io.err(text::VERSION_NUMBER_REQUIRED);
        return 3;
    }
    let version = version::strip_iojs_prefix(prefixed_version);
    let os = platform::get_os(nvm);
    if os.is_empty() {
        return 2;
    }

    // `local TMPDIR` with no value: empty in bash and zsh, the caller's
    // value in dash.
    let saved_tmpdir = nvm.local("TMPDIR");
    let mut tmpdir = nvm.vars.get("TMPDIR").to_string();
    nvm.restore("TMPDIR", saved_tmpdir);

    let name = if flavor == "node" { "node" } else { "io.js" };
    let progress = match nvm.vars.get("NVM_NO_PROGRESS") {
        "1" => "-sS",
        _ => "--progress-bar",
    };
    nvm.io
        .echo(&text::downloading_and_installing(name, version));
    let printed = nvm.capture(|nvm| {
        nvm.vars.set_quiet("PROGRESS_BAR", progress);
        download_artifact(nvm, flavor, "binary", release_type, version);
    });
    let tarball = printed.lines().last().unwrap_or("").to_string();
    if Path::new(&tarball).is_file() {
        tmpdir = format!("{}/files", dirname(&tarball));
    }

    if install_binary_extract(nvm, &[&os, prefixed_version, version, &tarball, &tmpdir]) == 0 {
        // `ALIAS` and `provided_version` are `nvm install`'s own variables,
        // which nvm reads from in here.
        let alias = nvm.vars.get("ALIAS").to_string();
        if !alias.is_empty() {
            let provided = nvm.vars.get("provided_version").to_string();
            crate::cmd::nvm(nvm, &["alias".to_string(), alias, provided]);
        }
        return 0;
    }

    if nosource == "1" {
        nvm.io.err(text::BINARY_FAILED_ABORTED);
        return 2;
    }
    nvm.io.err(text::BINARY_FAILED_TRYING_SOURCE);
    if !tmpdir.is_empty() {
        remove_work_dir(nvm, &tmpdir);
    }
    1
}

/// nvm's test for a whole number above zero, with its statuses: 4 for
/// nothing, 1 for zero, 3 for a leading minus, 2 for anything else that is
/// not a number.
///
/// nvm: nvm_is_natural_num
pub fn natural_num_status(text: &str) -> i32 {
    if text.is_empty() {
        return 4;
    }
    if text == "0" {
        return 1;
    }
    if text.starts_with('-') {
        return 3;
    }
    let digits = text.trim().trim_start_matches('+');
    if !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()) {
        0
    } else {
        2
    }
}

/// Sets `NVM_MAKE_JOBS`, from the number given or from how many processor
/// cores the machine has.
///
/// nvm: nvm_get_make_jobs
pub fn get_make_jobs(nvm: &mut Nvm, requested: &str) -> i32 {
    if natural_num_status(requested) == 0 {
        nvm.vars.assign("NVM_MAKE_JOBS", requested);
        nvm.io.echo(&text::make_jobs(requested));
        return 0;
    } else if !requested.is_empty() {
        nvm.vars.unset("NVM_MAKE_JOBS");
        nvm.io.err(&text::make_jobs_invalid(requested));
    }

    let text_of = |nvm: &mut Nvm, program: &str, args: &[&str]| {
        tools::run(nvm, program, args)
            .map(|ran| ran.text())
            .unwrap_or_default()
    };
    let lines_of = |nvm: &mut Nvm, program: &str, args: &[&str]| {
        tools::run(nvm, program, args)
            .map(|ran| String::from_utf8_lossy(&ran.stdout).lines().count())
            .unwrap_or(0)
            .to_string()
    };
    let cores = match platform::get_os(nvm).as_str() {
        "linux" => match fs::read_to_string("/proc/cpuinfo") {
            Ok(info) => {
                let processor = posix_re::ere("^processor.+: [0-9]+");
                info.lines()
                    .filter(|line| processor.as_ref().is_some_and(|re| re.is_match(line)))
                    .count()
                    .to_string()
            }
            Err(error) => {
                let reason = crate::text::messages::strerror(&error);
                nvm.io.err(&format!("grep: /proc/cpuinfo: {reason}"));
                String::new()
            }
        },
        "freebsd" | "darwin" | "openbsd" => text_of(nvm, "sysctl", &["-n", "hw.ncpu"]),
        "sunos" => lines_of(nvm, "psrinfo", &[]),
        "aix" => lines_of(nvm, "pmcycles", &["-m"]),
        _ => {
            // glibc, FreeBSD, macOS and Cygwin spell it with the underscore,
            // POSIX without. Windows shells inherit `NUMBER_OF_PROCESSORS`.
            let asked = [
                ("getconf", "_NPROCESSORS_ONLN"),
                ("getconf", "NPROCESSORS_ONLN"),
                ("nproc", ""),
            ]
            .into_iter()
            .find_map(|(program, arg)| {
                let args: &[&str] = if arg.is_empty() { &[] } else { &[arg] };
                tools::run_quiet(nvm, program, args).filter(|ran| ran.ok())
            });
            match asked {
                Some(ran) => ran.text(),
                None => nvm.vars.get("NUMBER_OF_PROCESSORS").to_string(),
            }
        }
    };

    if natural_num_status(&cores) != 0 {
        nvm.io.err(text::CORES_UNKNOWN);
        nvm.io.err(text::CORES_REPORT);
        nvm.vars.assign("NVM_MAKE_JOBS", "1");
    } else {
        nvm.io.echo(&text::cores_detected(&cores));
        let count: u64 = cores.trim().trim_start_matches('+').parse().unwrap_or(1);
        if count > 2 {
            let jobs = (count - 1).to_string();
            nvm.vars.assign("NVM_MAKE_JOBS", jobs.clone());
            nvm.io.echo(&text::make_threads(&jobs));
        } else {
            nvm.vars.assign("NVM_MAKE_JOBS", "1");
            nvm.io.echo(text::CORES_FEW);
        }
    }
    0
}

/// The version `clang --version` reports.
///
/// nvm: nvm_clang_version
pub fn clang_version(nvm: &mut Nvm) -> String {
    let output = tools::run(nvm, "clang", &["--version"])
        .map(|ran| ran.stdout)
        .unwrap_or_default();
    String::from_utf8_lossy(&output)
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line
                .split([' ', '\t'])
                .filter(|field| !field.is_empty())
                .collect();
            let at = if fields.get(1) == Some(&"version") {
                2
            } else if fields.get(2) == Some(&"version") {
                3
            } else {
                return None;
            };
            // awk prints an empty line for a field that is not there.
            let version = fields.get(at).copied().unwrap_or("");
            Some(version.split('-').next().unwrap_or("").to_string())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Downloads a release's source and builds it with `./configure` and
/// `make`. Returns 0 when it is installed.
///
/// nvm: nvm_install_source
pub fn install_source(
    nvm: &mut Nvm,
    flavor: &str,
    release_type: &str,
    prefixed_version: &str,
    make_jobs: &str,
    additional_parameters: &str,
) -> i32 {
    if !matches!(flavor, "node" | "iojs") {
        nvm.io.err(text::SUPPORTED_FLAVORS);
        return 4;
    }
    if prefixed_version.is_empty() {
        nvm.io.err(text::VERSION_NUMBER_REQUIRED);
        return 3;
    }
    let version = version::strip_iojs_prefix(prefixed_version);

    let mut additional = additional_parameters.to_string();
    let arch = platform::get_arch(nvm);
    if matches!(arch.as_str(), "armv6l" | "armv7l") {
        additional = match additional.is_empty() {
            true => "--without-snapshot".to_string(),
            false => format!("--without-snapshot {additional}"),
        };
    }
    if !additional.is_empty() {
        nvm.io.echo(&text::additional_options(&additional));
    }
    let os = platform::get_os(nvm);

    // Two more locals nvm declares without a value, which dash fills from
    // the caller.
    let saved_cxx = nvm.local("MAKE_CXX");
    let mut make_cxx = nvm.vars.get("MAKE_CXX").to_string();
    nvm.restore("MAKE_CXX", saved_cxx);
    let saved_override = nvm.local("MAKE_SHELL_OVERRIDE");
    let mut shell_override = nvm.vars.get("MAKE_SHELL_OVERRIDE").to_string();
    nvm.restore("MAKE_SHELL_OVERRIDE", saved_override);

    // Makefiles of Node before 0.12 have unquoted globs that zsh refuses.
    if version::greater("0.12.0", version) {
        shell_override = " SHELL=/bin/sh".to_string();
    }
    let or = |value: &str, default: &str| match value {
        "" => default.to_string(),
        value => value.to_string(),
    };
    let compilers = format!(
        "CC={} CXX={}",
        or(nvm.vars.get("CC"), "cc"),
        or(nvm.vars.get("CXX"), "c++")
    );
    let mut make = format!("make{shell_override}");
    match os.as_str() {
        "freebsd" | "openbsd" => {
            make = format!("gmake{shell_override}");
            make_cxx = compilers.clone();
        }
        "darwin" => make_cxx = compilers.clone(),
        "aix" => make = format!("gmake{shell_override}"),
        _ => {}
    }
    let path = nvm.vars.get("PATH").to_string();
    if paths::on_path(&path, "clang++")
        && paths::on_path(&path, "clang")
        && version::gte(&clang_version(nvm), "3.5")
        && (nvm.vars.get("CC").is_empty() || nvm.vars.get("CXX").is_empty())
    {
        nvm.io.echo(text::CLANG_DETECTED);
        make_cxx = compilers;
    }

    // nvm assigns PROGRESS_BAR without `local` here, so it stays set.
    let progress = match nvm.vars.get("NVM_NO_PROGRESS") {
        "1" => "-sS",
        _ => "--progress-bar",
    };
    nvm.vars.assign("PROGRESS_BAR", progress);

    let printed = nvm.capture(|nvm| {
        download_artifact(nvm, flavor, "source", release_type, version);
    });
    let tarball = printed.lines().last().unwrap_or("").to_string();
    if !Path::new(&tarball).is_file() {
        return 1;
    }
    let tmpdir = format!("{}/files", dirname(&tarball));

    let built = build(
        nvm,
        Build {
            os: &os,
            version,
            prefixed_version,
            tarball: &tarball,
            tmpdir: &tmpdir,
            additional: &additional,
            make: &make,
            make_jobs,
            make_cxx: &make_cxx,
        },
    );
    if !built {
        nvm.io.err(&text::install_failed(version));
        remove_work_dir(nvm, &tmpdir);
        return 1;
    }
    0
}

struct Build<'a> {
    os: &'a str,
    version: &'a str,
    prefixed_version: &'a str,
    tarball: &'a str,
    tmpdir: &'a str,
    additional: &'a str,
    make: &'a str,
    make_jobs: &'a str,
    make_cxx: &'a str,
}

/// The steps nvm runs in a subshell inside the unpacked source: configure,
/// make, make install. Stops at the first one that fails.
fn build(nvm: &mut Nvm, build: Build) -> bool {
    if !tools::mkdir_p(nvm, build.tmpdir)
        || extract_tarball(nvm, &[build.os, build.version, build.tarball, build.tmpdir]) != 0
    {
        return false;
    }
    let version_path = match paths::version_path(nvm, build.prefixed_version) {
        Ok(path) => path,
        Err(message) => {
            nvm.io.err(&message);
            return false;
        }
    };
    if !Path::new(build.tmpdir).is_dir() {
        return false;
    }

    let prefix = format!("--prefix={version_path}");
    let options: Vec<&str> = build.additional.split_whitespace().collect();
    // nvm shows the command between `$>` and `<`.
    let mut shown = vec!["$>./configure".to_string(), prefix.clone()];
    shown.extend(options.iter().map(|option| option.to_string()));
    match options.is_empty() {
        true => shown.push("<".to_string()),
        false => shown.last_mut().unwrap().push('<'),
    }
    nvm.io.echo(&shown.join(" "));

    // Each step is a command word in nvm's own code, so when one cannot be
    // started the shell says so, naming that line.
    let in_source = |nvm: &mut Nvm, site: &Site, program: &str, args: &[&str]| -> bool {
        if let Some(why) = complaint::cannot_start_in(nvm, build.tmpdir, program) {
            let line = complaint::not_started(nvm, site, program, why);
            nvm.io.err(&line);
            return false;
        }
        let Some(mut command) = tools::command(nvm, program) else {
            return false;
        };
        command
            .args(args)
            .current_dir(build.tmpdir)
            .env("PWD", build.tmpdir);
        tools::run_through(nvm, command) == 0
    };

    let mut configure = vec![prefix.as_str()];
    configure.extend(&options);
    if !in_source(nvm, &sites::BUILD_CONFIGURE, "./configure", &configure) {
        return false;
    }

    let mut make_words = build.make.split_whitespace();
    let make_program = make_words.next().unwrap_or("make");
    let mut make_args: Vec<&str> = make_words.collect();
    make_args.extend(["-j", build.make_jobs]);
    make_args.extend(build.make_cxx.split_whitespace());
    if !in_source(nvm, &sites::BUILD_MAKE, make_program, &make_args) {
        return false;
    }
    // `rm -f` of the version path: fine when nothing is there, a failure
    // when a directory is.
    match fs::symlink_metadata(&version_path) {
        Ok(meta) if meta.is_dir() => return false,
        Ok(_) => {
            if fs::remove_file(&version_path).is_err() {
                return false;
            }
        }
        Err(_) => {}
    }
    make_args.push("install");
    in_source(nvm, &sites::BUILD_MAKE_INSTALL, make_program, &make_args)
}

/// Whether an installed version has what it needs to run: a `bin/node`
/// that is not empty and can be executed, and an `npm` that is not a link
/// to nowhere. Node itself is not started.
///
/// nvm: nvm_validate_install
pub fn validate_install(nvm: &mut Nvm, version: &str) -> i32 {
    if version.is_empty() {
        return 1;
    }
    let Ok(version_path) = paths::version_path(nvm, version) else {
        return 1;
    };
    if version_path.is_empty() || !Path::new(&version_path).is_dir() {
        return 1;
    }
    let binary = match platform::get_os(nvm).as_str() {
        "win" => "node.exe",
        _ => "node",
    };
    let node = format!("{version_path}/bin/{binary}");
    let has_content = fs::metadata(&node).is_ok_and(|meta| meta.len() > 0);
    if !has_content || !nvm.backend.is_executable(&node) {
        nvm.io.err(&text::node_binary_missing(&node));
        return 1;
    }
    let npm = format!("{version_path}/bin/npm");
    let is_link = fs::symlink_metadata(&npm).is_ok_and(|meta| meta.file_type().is_symlink());
    if is_link && !Path::new(&npm).exists() {
        nvm.io.err(&text::npm_dangling(version));
        return 1;
    }
    0
}

/// Versions whose archive is in the download cache, matching a pattern,
/// one per line and sorted.
///
/// nvm: nvm_ls_cached
pub fn ls_cached(nvm: &mut Nvm, pattern: &str) -> String {
    let cache = cache_dir(nvm);
    let suffix = format!("-{}-{}", platform::get_os(nvm), platform::get_arch(nvm));
    let names = |dir: &str| -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| !name.starts_with('.'))
            .collect();
        names.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        names
    };
    // `^\(node\|iojs\)-v[0-9][0-9.]*$`
    let is_release = |name: &str| {
        name.strip_prefix("node-")
            .or_else(|| name.strip_prefix("iojs-"))
            .and_then(|rest| rest.strip_prefix('v'))
            .is_some_and(|numbers| {
                numbers.starts_with(|c: char| c.is_ascii_digit())
                    && numbers.chars().all(|c| c.is_ascii_digit() || c == '.')
            })
    };

    let mut found: Vec<String> = Vec::new();
    let bin = format!("{cache}/bin");
    if Path::new(&bin).is_dir() {
        found.extend(
            names(&bin)
                .iter()
                .filter_map(|name| name.strip_suffix(&suffix))
                .filter(|name| is_release(name))
                .map(str::to_string),
        );
    }
    let src = format!("{cache}/src");
    if Path::new(&src).is_dir() {
        found.extend(names(&src).into_iter().filter(|name| is_release(name)));
    }

    let wanted = posix_re::bre(&version::ensure_prefix(pattern));
    let mut lines: Vec<String> = found
        .into_iter()
        .map(|name| match name.strip_prefix("node-") {
            Some(rest) => rest.to_string(),
            None => name,
        })
        .filter(|line| wanted.as_ref().is_some_and(|re| re.is_match(line)))
        .collect();
    local::sort_versions(&mut lines);
    lines.join("\n")
}

/// Resolves a pattern to one version without the network: an installed
/// version first, then one in the download cache. `N/A` with status 3 when
/// neither has it.
///
/// nvm: nvm_offline_version
pub fn offline_version(nvm: &mut Nvm, pattern: &str) -> (String, i32) {
    let (version, _) = local::version(nvm, pattern);
    if version != "N/A" {
        return (version, 0);
    }
    let cached = ls_cached(nvm, pattern);
    match cached.lines().last() {
        Some(version) if !version.is_empty() => (version.to_string(), 0),
        _ => ("N/A".to_string(), 3),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirname_matches_the_program() {
        assert_eq!(dirname("/a/b/c.tar.xz"), "/a/b");
        assert_eq!(dirname("/a"), "/");
        assert_eq!(dirname("a"), ".");
        assert_eq!(dirname("a/b/"), "a");
        assert_eq!(dirname(""), ".");
    }

    #[test]
    fn natural_numbers_and_their_statuses() {
        assert_eq!(natural_num_status("8"), 0);
        assert_eq!(natural_num_status(""), 4);
        assert_eq!(natural_num_status("0"), 1);
        assert_eq!(natural_num_status("-1"), 3);
        assert_eq!(natural_num_status("four"), 2);
        assert_eq!(natural_num_status("1.5"), 2);
    }
}
