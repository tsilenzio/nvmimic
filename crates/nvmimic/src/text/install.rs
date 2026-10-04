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

// nvm: nvm_download_artifact nvm_install_binary nvm_install_source
pub const VERSION_NUMBER_REQUIRED: &str = "A version number is required.";
pub const VERSION_DISALLOWED: &str = "Invalid version: contains disallowed characters";
pub const CHECKSUM_CHECK_FAILED: &str = "Checksum check failed!";
pub const REMOVING_BROKEN_CACHE: &str = "Removing the broken local cache...";

pub fn no_binary(version: &str) -> String {
    format!("No precompiled binary available for {version}.")
}

pub fn offline_using(path: &str) -> String {
    format!("Offline: using cached archive {path}")
}

pub fn offline_missing(slug: &str) -> String {
    format!("Offline: no cached archive found for {slug}")
}

pub fn creating_directory_failed(dir: &str) -> String {
    format!("creating directory {dir} failed")
}

pub fn local_cache_found(path: &str) -> String {
    format!("Local cache found: {path}")
}

pub fn using_existing_archive(path: &str) -> String {
    format!("Checksums match! Using existing downloaded archive {path}")
}

pub fn downloading(url: &str) -> String {
    format!("Downloading {url}...")
}

pub fn download_failed(url: &str) -> String {
    format!("download from {url} failed")
}

pub fn http_404(url: &str) -> String {
    format!("HTTP 404 at URL {url}")
}

// nvm: nvm_extract_tarball nvm_install_binary_extract
pub const EXTRACT_ARGUMENTS: &str = "nvm_extract_tarball requires exactly 4 arguments";
pub const BINARY_EXTRACT_ARGUMENTS: &str = "nvm_install_binary_extract needs 5 parameters";

// nvm: nvm_install_binary
pub const BINARY_FAILED_ABORTED: &str = "Binary download failed. Download from source aborted.";
pub const BINARY_FAILED_TRYING_SOURCE: &str = "Binary download failed, trying source.";

pub fn downloading_and_installing(what: &str, version: &str) -> String {
    format!("Downloading and installing {what} {version}...")
}

// nvm: nvm_install_source
pub const CLANG_DETECTED: &str =
    "Clang v3.5+ detected! CC or CXX not specified, will use Clang as C/C++ compiler!";

pub fn additional_options(options: &str) -> String {
    format!("Additional options while compiling: {options}")
}

pub fn install_failed(version: &str) -> String {
    format!("nvm: install {version} failed!")
}

// nvm: nvm_install_npm_if_needed
pub const INSTALLING_NPM: &str = "Installing npm...";
pub const NPM_NEEDS_NODE: &str = "npm requires node v0.2.3 or higher";

// nvm: nvm:install
pub const TRIPLE_DASH: &str = "arguments with `---` are not supported - this is likely a typo";
pub const SOURCE_AND_BINARY: &str =
    "-s and -b cannot be set together since they would skip install from both binary and source";
pub const DEFAULT_AND_ALIAS: &str =
    "--default and --alias are mutually exclusive, and may not be provided more than once";
pub const REINSTALL_ONCE: &str = "--reinstall-packages-from may not be provided more than once";
pub const REINSTALL_NEEDS_VERSION: &str =
    "If --reinstall-packages-from is provided, it must point to an installed version of node.";
pub const REINSTALL_ONCE_OR_COPY: &str = "--reinstall-packages-from may not be provided more than once, or combined with `--copy-packages-from`";
pub const COPY_NEEDS_VERSION: &str =
    "If --copy-packages-from is provided, it must point to an installed version of node.";
pub const SOURCE_WITH_NO_FALLBACK: &str = "-s cannot be combined with NVM_NO_SOURCE_FALLBACK=1 since that would skip install from both binary and source";
pub const NEEDS_CURL_OR_WGET: &str = "nvm needs curl or wget to proceed.";
pub const INSTALLING_LATEST_LTS: &str = "Installing latest LTS version.";
pub const INSTALL_USAGE: [&str; 3] = [
    "Usage: nvm install [<version>]",
    "  Provide a <version>, or run from a directory containing an .nvmrc file.",
    "  Run `nvm --help` for full help.",
];
pub const SAME_VERSION_REINSTALL: &str =
    "You can't reinstall global packages from the same version of node you're installing.";
pub const HOOK_DISPATCH: &str = "** $NVM_INSTALL_THIRD_PARTY_HOOK env var set; dispatching to third-party installation method **";
pub const HOOK_FAILED: &str =
    "*** Third-party $NVM_INSTALL_THIRD_PARTY_HOOK env var failed to install! ***";
pub const HOOK_DID_NOT_INSTALL: &str = "*** Third-party $NVM_INSTALL_THIRD_PARTY_HOOK env var claimed to succeed, but failed to install! ***";
pub const NO_FREEBSD_BINARY: &str = "Currently, there is no binary for FreeBSD";
pub const NO_OPENBSD_BINARY: &str = "Currently, there is no binary for OpenBSD";
pub const NO_SOURCE_ON_WINDOWS: &str = "Installing from source on non-WSL Windows is not supported";

pub fn needs_equals(option: &str) -> String {
    format!("If {option} is provided, it must point to an installed version of node using `=`.")
}

pub fn installing_lts_line(lts: &str) -> String {
    format!("Installing with latest version of LTS line: {lts}")
}

pub fn lts_alias_not_local(lts: &str) -> String {
    format!(
        "LTS alias '{lts}' not found locally. Run `nvm ls-remote --lts` first to populate LTS aliases."
    )
}

pub fn lts_version_not_found(lts: &str, command: &str) -> String {
    format!(
        "Version with LTS filter '{lts}' not found - try `{command}` to browse available versions."
    )
}

/// `filter` is empty or ends in a space, as nvm builds it.
pub fn version_not_found_offline(version: &str, filter: &str, command: &str) -> String {
    format!(
        "Version '{version}' {filter}not found locally or in cache - try `{command}` to browse available versions."
    )
}

pub fn version_not_found(version: &str, filter: &str, command: &str) -> String {
    format!("Version '{version}' {filter}not found - try `{command}` to browse available versions.")
}

pub fn lts_filter(lts: &str) -> String {
    format!("(with LTS filter '{lts}') ")
}

pub const LTS_FILTER_ANY: &str = "(with LTS filter) ";

pub fn already_installed(version: &str) -> String {
    format!("{version} is already installed.")
}

pub fn no_sunos_binary(version: &str) -> String {
    format!("Currently, there is no binary of version {version} for SunOS")
}

pub fn binary_not_available(version: &str) -> String {
    format!("Binary download is not available for {version}")
}

pub fn failed_verification(version: &str) -> String {
    format!("The install of {version} reported success but failed verification; not activating it.")
}

// nvm: nvm:uninstall
pub const UNINSTALL_USAGE: [&str; 4] = [
    "Usage: nvm uninstall <version>",
    "       nvm uninstall --lts",
    "       nvm uninstall --lts=<LTS name>",
    "  Run `nvm --help` for full help.",
];
pub const UNINSTALL_PERMISSIONS: &str =
    "Cannot uninstall, incorrect permissions on installation folder.";
pub const UNINSTALL_PERMISSIONS_HINT: &str = "This is usually caused by running `npm install -g` as root. Run the following commands as root to fix the permissions and then try again.";

pub fn cannot_uninstall_active_iojs(version: &str, pattern: &str) -> String {
    format!(
        "nvm: Cannot uninstall currently-active io.js version, {version} (inferred from {pattern})."
    )
}

pub fn cannot_uninstall_active_node(version: &str, pattern: &str) -> String {
    format!(
        "nvm: Cannot uninstall currently-active node version, {version} (inferred from {pattern})."
    )
}

pub fn inferred_not_installed(version: &str, pattern: &str) -> String {
    format!("Version '{version}' (inferred from {pattern}) is not installed.")
}

pub fn version_not_installed(version: &str) -> String {
    format!("Version '{version}' is not installed.")
}

pub fn uninstalled_iojs(version: &str) -> String {
    format!("Uninstalled io.js {version}")
}

pub fn uninstalled_node(version: &str) -> String {
    format!("Uninstalled node {version}")
}

pub fn chown_hint(user: &str, path: &str) -> String {
    format!("  chown -R {user} \"{path}\"")
}

pub fn chmod_hint(path: &str) -> String {
    format!("  chmod -R u+w \"{path}\"")
}

// nvm: nvm_install_default_packages
pub const DEFAULT_PACKAGES_FAILED: &str = "Failed installing default packages. Please check if your default-packages file or a package in it has problems!";

pub fn installing_default_packages(nvm_dir: &str) -> String {
    format!("Installing default global packages from {nvm_dir}/default-packages...")
}

pub fn npm_install_quiet(packages: &str) -> String {
    format!("npm install -g --quiet {packages}")
}

// nvm: nvm:install-latest-npm nvm:reinstall-packages
pub const INSTALL_LATEST_NPM_USAGE: [&str; 2] = [
    "Usage: nvm install-latest-npm",
    "  Run `nvm --help` for full help.",
];
pub const RUN_HELP: &str = "  Run `nvm --help` for full help.";
pub const REINSTALL_FROM_CURRENT: &str =
    "Can not reinstall packages from the current version of node.";
pub const NO_SYSTEM_NODE_OR_IOJS: &str = "No system version of node or io.js detected.";
pub const NO_GLOBAL_PACKAGES: &str = "No installed global packages found...";
pub const NO_LINKED_PACKAGES: &str = "No linked global packages found...";

pub fn reinstall_usage(command: &str) -> String {
    format!("Usage: nvm {command} <version>")
}

pub fn reinstalling_from(version: &str) -> String {
    format!("Reinstalling global packages from {version}...")
}

pub fn linking_from(version: &str) -> String {
    format!("Linking global packages from {version}...")
}

// nvm: nvm_install_latest_npm
pub const UPGRADING_NPM: &str = "Attempting to upgrade to the latest working version of npm...";
pub const NO_NODE_VERSION: &str = "Unable to obtain node version.";
pub const NO_NPM_VERSION: &str = "Unable to obtain npm version.";

pub fn detected_versions(node: &str, npm: &str) -> String {
    format!("Detected node version {node}, npm version v{npm}")
}

pub fn npm_upgraded_to(version: &str) -> String {
    format!("* npm upgraded to: v{version}")
}

// nvm: nvm_get_latest
pub const LATEST_NO_REDIRECT: &str =
    "https://latest.nvm.sh did not redirect to the latest release on GitHub";
