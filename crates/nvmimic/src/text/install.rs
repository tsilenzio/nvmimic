//! Messages from nvm's install, uninstall and cache code, copied exactly
//! from nvm.sh. `cargo xtask string-audit` checks them against the pinned
//! nvm.sh like the rest of this directory.
//
// Text in this file is from nvm (https://github.com/nvm-sh/nvm), MIT licensed:
// Copyright (c) 2010 Tim Caswell, Copyright (c) 2014 Jordan Harband.

// nvm: nvm_get_checksum_binary
pub const NO_SHA256_TOOL: &str =
    "Unaliased sha256sum, shasum, sha256, gsha256sum, openssl, or bssl not found.";
pub const NO_SHA1_TOOL: &str = "Unaliased sha1sum or sha1 not found.";

// nvm: nvm_compute_checksum nvm_compare_checksum
pub const CHECKSUM_FILE_EMPTY: &str = "Provided file to checksum is empty.";
pub const CHECKSUM_FILE_MISSING: &str = "Provided file to checksum does not exist.";
pub const CHECKSUM_EMPTY: &str = "Provided checksum to compare to is empty.";
pub const CHECKSUM_UNVERIFIED: &str = "WARNING: Continuing *without checksum verification*";
pub const CHECKSUMS_MATCHED: &str = "Checksums matched!";

pub fn computing_checksum(how: &str) -> String {
    format!("Computing checksum with {how}")
}

pub fn computed_checksum_empty(file: &str) -> String {
    format!("Computed checksum of '{file}' is empty.")
}

pub fn checksums_differ(computed: &str, expected: &str) -> String {
    format!("Checksums do not match: '{computed}' found, '{expected}' expected.")
}

// nvm: nvm_get_checksum nvm_get_download_slug nvm_download_artifact
pub const SUPPORTED_FLAVORS: &str = "supported flavors: node, iojs";
pub const SUPPORTED_KINDS: &str = "supported kinds: binary, source";

// nvm: nvm_get_minor_version
pub const VERSION_REQUIRED: &str = "a version is required";
pub const INVALID_VERSION_NUMBER: &str = "invalid version number";
pub const INVALID_VERSION_REPORT: &str = "invalid version number! (please report this)";

// nvm: nvm_ensure_default_set
pub const DEFAULT_SET_VERSION_REQUIRED: &str = "nvm_ensure_default_set: a version is required";

pub fn creating_default_alias(output: &str) -> String {
    format!("Creating default alias: {output}")
}

// nvm: nvm_get_make_jobs
pub const CORES_UNKNOWN: &str =
    "Can not determine how many core(s) are available, running in single-threaded mode.";
pub const CORES_REPORT: &str =
    "Please report an issue on GitHub to help us make nvm run faster on your computer!";
pub const CORES_FEW: &str =
    "Number of CPU core(s) less than or equal to 2, running in single-threaded mode";

pub fn make_jobs(jobs: &str) -> String {
    format!("number of `make` jobs: {jobs}")
}

pub fn make_jobs_invalid(value: &str) -> String {
    format!("{value} is invalid for number of `make` jobs, must be a natural number")
}

pub fn cores_detected(cores: &str) -> String {
    format!("Detected that you have {cores} CPU core(s)")
}

pub fn make_threads(jobs: &str) -> String {
    format!("Running with {jobs} threads to speed up the build")
}

// nvm: nvm_validate_install
pub fn node_binary_missing(path: &str) -> String {
    format!("The installed node binary at {path} is missing or empty.")
}

pub fn npm_dangling(version: &str) -> String {
    format!("npm for {version} is a dangling symlink.")
}

// nvm: nvm_check_file_permissions
pub fn not_writable(path: &str) -> String {
    format!("file is not writable or self-owned: {path}")
}

// nvm: nvm_acquire_install_lock
pub fn removing_stale_lock(version: &str, minutes: &str) -> String {
    format!("Removing stale install lock for {version} (older than {minutes} minute(s))")
}

pub fn lock_timed_out(seconds: &str, version: &str) -> String {
    format!("Timed out after {seconds}s waiting for another install of {version} to finish.")
}

pub fn lock_remove_hint(lock: &str) -> String {
    format!("If no other install is running, remove {lock} and try again.")
}

pub fn lock_waiting(version: &str) -> String {
    format!("Waiting for another install of {version} to finish...")
}

// nvm: nvm_get_default_packages
pub fn one_package_per_line(file: &str) -> String {
    format!(
        "Only one package per line is allowed in `{file}`. Please remove any lines with multiple space-separated values."
    )
}

// nvm: nvm:cache
pub const CACHE_CLEARED: &str = "nvm cache cleared.";
pub const CACHE_USAGE: &[&str] = &[
    "Usage: nvm cache dir",
    "       nvm cache clear",
    "  Run `nvm --help` for full help.",
];

pub fn cache_clear_failed(dir: &str) -> String {
    format!("Unable to clear nvm cache: {dir}")
}
