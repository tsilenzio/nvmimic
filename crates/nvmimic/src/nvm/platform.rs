//! Which operating system and processor this is, as nvm names them, and
//! what each kind of Node release exists for.

use super::complaint;
use super::tools;
use super::version;
use crate::context::Nvm;
use crate::text::install as text;
use std::path::Path;

/// The operating system, asked once per call of nvmimic. nvm asks `uname`
/// again every time it needs to know.
///
/// nvm: nvm_get_os
pub fn get_os(nvm: &mut Nvm) -> String {
    if let Some(os) = nvm.cache.borrow().os.clone() {
        return os;
    }
    // With no `uname` the shell complains every time nvm asks, so that
    // answer is not kept.
    if let Some(why) = complaint::cannot_start(nvm, "uname") {
        let line = complaint::not_started(nvm, &complaint::sites::GET_OS_UNAME, "uname", why);
        nvm.io.err(&line);
        return String::new();
    }
    let os = find_os(nvm);
    nvm.cache.borrow_mut().os = Some(os.clone());
    os
}

fn find_os(nvm: &mut Nvm) -> String {
    let uname = tools::run(nvm, "uname", &["-a"])
        .map(|ran| ran.text())
        .unwrap_or_default();
    let os = if uname.starts_with("Linux ") {
        "linux"
    } else if uname.starts_with("Darwin ") {
        "darwin"
    } else if uname.starts_with("SunOS ") {
        "sunos"
    } else if uname.starts_with("FreeBSD ") {
        "freebsd"
    } else if uname.starts_with("OpenBSD ") {
        "openbsd"
    } else if uname.starts_with("AIX ") {
        "aix"
    } else if ["CYGWIN", "MSYS", "MINGW"]
        .iter()
        .any(|prefix| uname.starts_with(prefix))
    {
        "win"
    } else {
        ""
    };
    os.to_string()
}

/// The processor, asked once per call of nvmimic like [`get_os`].
///
/// nvm: nvm_get_arch
pub fn get_arch(nvm: &mut Nvm) -> String {
    if let Some(arch) = nvm.cache.borrow().arch.clone() {
        return arch;
    }
    let arch = find_arch(nvm);
    nvm.cache.borrow_mut().arch = Some(arch.clone());
    arch
}

fn find_arch(nvm: &mut Nvm) -> String {
    let os = get_os(nvm);
    let mut long_bit = String::new();
    let mut host = if os == "sunos" {
        // pkgsrc knows best. Without it, the kernel's instruction set.
        match tools::run(nvm, "pkg_info", &["-Q", "MACHINE_ARCH", "pkg_install"]) {
            Some(ran) if ran.ok() => ran.text().lines().last().unwrap_or("").to_string(),
            _ => tools::run(nvm, "isainfo", &["-n"])
                .map(|ran| ran.text())
                .unwrap_or_default(),
        }
    } else if os == "aix" {
        "ppc64".to_string()
    } else {
        let machine = tools::run(nvm, "uname", &["-m"])
            .map(|ran| ran.text())
            .unwrap_or_default();
        long_bit = tools::run_quiet(nvm, "getconf", &["LONG_BIT"])
            .map(|ran| ran.text())
            .unwrap_or_default();
        machine
    };

    let mut arch = match host.as_str() {
        "x86_64" | "amd64" => "x64".to_string(),
        "aarch64" | "armv8l" => "arm64".to_string(),
        "loongarch64" => "loong64".to_string(),
        other if other.len() == 4 && other.starts_with('i') && other.ends_with("86") => {
            "x86".to_string()
        }
        other => other.to_string(),
    };

    // A 32-bit container on a 64-bit kernel.
    if long_bit == "32" && arch == "x64" {
        arch = "x86".to_string();
    }

    // A 64-bit ARM kernel under a 32-bit userland: `/sbin/init` is a 32-bit program.
    if arch == "arm64"
        && tools::run(nvm, "uname", &[]).is_some_and(|ran| ran.text() == "Linux")
        && std::fs::read("/sbin/init").is_ok_and(|bytes| bytes.get(4) == Some(&1))
    {
        arch = "armv7l".to_string();
        host = arch.clone();
    }
    let _ = host;

    if os == "linux" && Path::new("/etc/alpine-release").is_file() {
        arch = match arch.as_str() {
            "x64" => "x64-musl".to_string(),
            "arm64" => "arm64-musl".to_string(),
            _ => arch,
        };
    }
    arch
}

// The functions below return nvm's exit status: 0 for yes. The other
// values are the ones nvm's version comparison leaves behind.

/// nvm: nvm_is_merged_node_version
pub fn is_merged_node_version(version: &str) -> i32 {
    version::gte_status(version, "v4.0.0")
}

/// Binaries started with Node 0.8.6.
///
/// nvm: nvm_binary_available
pub fn binary_available(version: &str) -> i32 {
    version::gte_status(version::strip_iojs_prefix(version), "v0.8.6")
}

/// io.js has had Solaris binaries since 3.3.1.
///
/// nvm: nvm_iojs_version_has_solaris_binary
pub fn iojs_version_has_solaris_binary(version: &str) -> i32 {
    let stripped = version::strip_iojs_prefix(version);
    if stripped == version {
        return 1;
    }
    version::gte_status(stripped, "v3.3.1")
}

/// Node before the io.js merge had Solaris binaries from 0.8.6 on.
///
/// nvm: nvm_node_version_has_solaris_binary
pub fn node_version_has_solaris_binary(version: &str) -> i32 {
    if version::strip_iojs_prefix(version) != version {
        return 1;
    }
    match version::gte_status(version, "v0.8.6") {
        0 => (version::gte_status(version, "v1.0.0") == 0) as i32,
        status => status,
    }
}

/// nvm: nvm_has_solaris_binary
pub fn has_solaris_binary(version: &str) -> i32 {
    if is_merged_node_version(version) == 0 {
        0
    } else if version::is_iojs(version) {
        iojs_version_has_solaris_binary(version)
    } else {
        node_version_has_solaris_binary(version)
    }
}

/// Whether releases of this version come as `.tar.xz` and this machine can
/// unpack one.
///
/// nvm: nvm_supports_xz
pub fn supports_xz(nvm: &mut Nvm, version: &str) -> i32 {
    if version.is_empty() {
        return 1;
    }
    let os = get_os(nvm);
    if os == "darwin" {
        let macos = tools::run(nvm, "sw_vers", &["-productVersion"])
            .map(|ran| ran.text())
            .unwrap_or_default();
        if version::greater("10.9.0", &macos) {
            return 1;
        }
    } else if os == "freebsd" {
        if !Path::new("/usr/lib/liblzma.so").exists() {
            return 1;
        }
    } else if !tools::run_quiet(nvm, "which", &["xz"]).is_some_and(|ran| ran.ok()) {
        return 1;
    }

    if is_merged_node_version(version) == 0 {
        return 0;
    }
    if version::gte(version, "0.12.10") && version::greater("0.13.0", version) {
        return 0;
    }
    if version::gte(version, "0.10.42") && version::greater("0.11.0", version) {
        return 0;
    }
    match os.as_str() {
        "darwin" => version::gte_status(version, "2.3.2"),
        _ => version::gte_status(version, "1.0.0"),
    }
}

/// The `major.minor` of a version, or nvm's error status.
///
/// nvm: nvm_get_minor_version
pub fn get_minor_version(nvm: &mut Nvm, version: &str) -> Result<String, i32> {
    if version.is_empty() {
        nvm.io.err(text::VERSION_REQUIRED);
        return Err(1);
    }
    let plain = |c: char| c == '.' || c.is_ascii_digit();
    let rest = version.strip_prefix('v');
    let bad = version == "v"
        || version.starts_with('.')
        || version.contains("..")
        || rest.is_some_and(|rest| !rest.chars().all(plain))
        || (rest.is_none() && !version.chars().all(plain))
        || rest.is_some_and(|rest| !rest.starts_with(|c: char| c.is_ascii_digit()));
    if bad {
        nvm.io.err(text::INVALID_VERSION_NUMBER);
        return Err(2);
    }
    let formatted = version::format(version);
    let minor: String = formatted
        .strip_prefix('v')
        .map(|rest| rest.split('.').take(2).collect::<Vec<_>>().join("."))
        .unwrap_or_default();
    if minor.is_empty() {
        nvm.io.err(text::INVALID_VERSION_REPORT);
        return Err(3);
    }
    Ok(minor)
}
