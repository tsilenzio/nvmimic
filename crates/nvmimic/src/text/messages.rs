//! Messages nvm prints, copied exactly from nvm.sh, grouped by the nvm
//! function that prints them. `cargo xtask string-audit` checks that each
//! fixed piece still appears in the pinned nvm.sh.
//
// Text in this file is from nvm (https://github.com/nvm-sh/nvm), MIT licensed:
// Copyright (c) 2010 Tim Caswell, Copyright (c) 2014 Jordan Harband.

// nvm: nvm_nvmrc_invalid_msg
pub const NVMRC_INVALID: &str = "invalid .nvmrc!
all non-commented content (anything after # is a comment) must be either:
  - a single bare nvm-recognized version-ish
  - or, multiple distinct key-value pairs, each key/value separated by a single equals sign (=)

additionally, a single bare nvm-recognized version-ish must be present (after stripping comments).";
pub const NVMRC_PARSED: &str = "non-commented content parsed:";

// nvm: nvm_rc_version
pub const NO_VERSION_NO_NVMRC: &str = "No version provided and no .nvmrc file found";

pub fn empty_nvmrc(path: &str) -> String {
    format!("Warning: empty .nvmrc file found at \"{path}\"")
}

pub fn found_nvmrc(path: &str, version: &str) -> String {
    format!("Found '{path}' with version <{version}>")
}

// nvm: nvm_normalize_lts
pub const TOO_MANY_LTS: &str = "That many LTS releases do not exist yet.";
pub const LTS_LOWERCASE: &str = "LTS names must be lowercase";

// nvm: nvm_alias
pub const ALIAS_REQUIRED: &str = "An alias is required.";
pub const ALIAS_MISSING: &str = "Alias does not exist.";

pub fn invalid_alias_name(name: &str) -> String {
    format!("invalid alias name: {name}")
}

pub fn alias_unreadable(path: &str) -> String {
    format!("Alias file is not readable: {path}")
}

// nvm: nvm_validate_implicit_alias
pub fn only_implicit_aliases() -> String {
    let (iojs, node) = (
        crate::nvm::version::IOJS_PREFIX,
        crate::nvm::version::NODE_PREFIX,
    );
    format!("Only implicit aliases 'stable', 'unstable', '{iojs}', and '{node}' are supported.")
}

// nvm: nvm_make_alias
pub const ALIAS_NAME_REQUIRED: &str = "an alias name is required";
pub const ALIAS_TARGET_REQUIRED: &str = "an alias target version is required";

// nvm: nvm_version_path
pub fn invalid_version(version: &str) -> String {
    format!("invalid version: {version}")
}
pub const VERSION_REQUIRED: &str = "version is required";

// nvm: nvm_ensure_version_installed
pub const NO_SYSTEM_VERSION: &str = "N/A: no system version of node/io.js is installed.";

pub fn not_installed_alias(provided: &str, version: &str) -> String {
    format!("N/A: version \"{provided} -> {version}\" is not yet installed.")
}

pub fn not_installed(version: &str) -> String {
    format!("N/A: version \"{version}\" is not yet installed.")
}

pub const LTS_IS_NOT_AN_ALIAS: &str = "`lts` is not an alias - you may need to run `nvm install --lts` to install and `nvm use --lts` to use it.";

pub fn run_install(provided: &str) -> String {
    format!("You need to run `nvm install {provided}` to install and use it.")
}

pub const RUN_INSTALL_NVMRC: &str =
    "You need to run `nvm install` to install and use the node version specified in `.nvmrc`.";

// nvm: nvm:deactivate
pub fn could_not_find_bin(nvm_dir: &str) -> String {
    format!("Could not find {nvm_dir}/*/bin in ${{PATH}}")
}

pub fn removed_bin(nvm_dir: &str) -> String {
    format!("{nvm_dir}/*/bin removed from ${{PATH}}")
}

pub fn could_not_find_man(nvm_dir: &str) -> String {
    format!("Could not find {nvm_dir}/*/share/man in ${{MANPATH}}")
}

pub fn removed_man(nvm_dir: &str) -> String {
    format!("{nvm_dir}/*/share/man removed from ${{MANPATH}}")
}

pub fn removed_node_modules(nvm_dir: &str) -> String {
    format!("{nvm_dir}/*/lib/node_modules removed from ${{NODE_PATH}}")
}

// nvm: nvm:use
pub const SAVE_ONCE: &str = "--save and -w may only be provided once";
pub const SEE_HELP_NVMRC: &str =
    "Please see `nvm --help` or https://github.com/nvm-sh/nvm#nvmrc for more information.";
pub const USE_USAGE: [&str; 3] = [
    "Usage: nvm use [<version>]",
    "  Provide a <version>, or run from a directory containing an .nvmrc file.",
    "  Run `nvm --help` for full help.",
];
pub const SYSTEM_NOT_FOUND: &str = "System version of node not found.";

pub fn now_using_system_node(version: &str, npm: &str) -> String {
    format!("Now using system version of node: {version}{npm}")
}

pub fn now_using_system_iojs(version: &str, npm: &str) -> String {
    format!("Now using system version of io.js: {version}{npm}")
}

pub fn infinite_loop(alias: &str) -> String {
    format!("The alias \"{alias}\" leads to an infinite loop. Aborting.")
}

pub fn now_using_iojs(version: &str, npm: &str) -> String {
    format!("Now using io.js {version}{npm}")
}

pub fn now_using_node(version: &str, npm: &str) -> String {
    format!("Now using node {version}{npm}")
}

// nvm: nvm_print_npm_version
pub fn npm_suffix(version: &str) -> String {
    format!(" (npm v{version})")
}

// nvm: nvm_write_nvmrc
pub fn wrote_nvmrc(version: &str) -> String {
    format!("Wrote version number ({version}) to .nvmrc")
}

pub fn unable_to_write_nvmrc(version: &str) -> String {
    format!("Warning: Unable to write version number ({version}) to .nvmrc")
}

// nvm: nvm_die_on_prefix
pub fn prefix_env(prefix: &str) -> String {
    format!(
        "nvm is not compatible with the \"PREFIX\" environment variable: currently set to \"{prefix}\""
    )
}
pub const UNSET_PREFIX: &str = "Run `unset PREFIX` to unset it.";

pub fn config_prefix_env(name: &str, value: &str) -> String {
    format!(
        "nvm is not compatible with the \"{name}\" environment variable: currently set to \"{value}\""
    )
}

pub fn unset_env(name: &str) -> String {
    format!("Run `unset {name}` to unset it.")
}

pub fn builtin_npmrc(path: &str) -> String {
    format!("Your builtin npmrc file ({path})")
}

pub fn global_npmrc(path: &str) -> String {
    format!("Your global npmrc file ({path})")
}

pub fn user_npmrc(path: &str) -> String {
    format!("Your user’s .npmrc file ({path})")
}

pub fn project_npmrc(path: &str) -> String {
    format!("Your project npmrc file ({path})")
}

pub const NPMRC_INCOMPATIBLE: &str =
    "has a `globalconfig` and/or a `prefix` setting, which are incompatible with nvm.";

pub fn run_to_unset(command: &str) -> String {
    format!("Run `{command}` to unset it.")
}

// nvm: nvm:exec nvm:run nvm:ls nvm:ls-remote nvm:alias nvm:version-remote
pub fn unsupported_option(option: &str) -> String {
    format!("Unsupported option \"{option}\".")
}

// nvm: nvm:ls
pub const NO_ALIAS_WITH_PATTERN: &str = "`--no-alias` is not supported when a pattern is provided.";

// nvm: nvm:which
pub const WHICH_USAGE: [&str; 3] = [
    "Usage: nvm which [current | <version>]",
    "  Provide a <version>, or run from a directory containing an .nvmrc file.",
    "  Run `nvm --help` for full help.",
];

// nvm: nvm:alias
pub const ALIAS_COMMENT: &str = "Aliases with a comment delimiter (#) are not supported.";
pub const ALIAS_SUBDIRECTORY: &str = "Aliases in subdirectories are not supported.";

pub fn version_does_not_exist(target: &str) -> String {
    format!("! WARNING: Version '{target}' does not exist.")
}

// nvm: nvm:unalias
pub const UNALIAS_USAGE: [&str; 2] = [
    "Usage: nvm unalias <name>",
    "  Run `nvm --help` for full help.",
];

pub fn builtin_alias(name: &str) -> String {
    format!("{name} is a default (built-in) alias and cannot be deleted.")
}

pub fn alias_does_not_exist(name: &str) -> String {
    format!("Alias {name} doesn't exist!")
}

pub fn deleted_alias(name: &str, original: &str) -> String {
    format!("Deleted alias {name} - restore it with `nvm alias \"{name}\" \"{original}\"`")
}

// nvm: nvm:exec
pub const EXEC_NO_VERSION: [&str; 3] = [
    "WARNING: `nvm exec` was invoked without a version argument and without an .nvmrc file.",
    "  Falling back to the active node version; this will become an error in a future release.",
    "  Pass `current` explicitly (e.g. `nvm exec current ...`) to silence this warning.",
];

pub fn running_latest_lts(version: &str, npm: &str) -> String {
    format!("Running node latest LTS -> {version}{npm}")
}

pub fn running_lts(lts: &str, version: &str, npm: &str) -> String {
    format!("Running node LTS \"{lts}\" -> {version}{npm}")
}

pub fn running_iojs(version: &str, npm: &str) -> String {
    format!("Running io.js {version}{npm}")
}

pub fn running_node(version: &str, npm: &str) -> String {
    format!("Running node {version}{npm}")
}

// nvm: nvm:run
pub const RUN_USAGE: [&str; 3] = [
    "Usage: nvm run [<version>] [<args>]",
    "  Provide a <version>, or run from a directory containing an .nvmrc file.",
    "  Run `nvm --help` for full help.",
];
pub const RUN_NO_VERSION: [&str; 3] = [
    "WARNING: `nvm run` was invoked without a version argument and without an .nvmrc file.",
    "  Falling back to the active node version; this will become an error in a future release.",
    "  Pass `current` explicitly (e.g. `nvm run current ...`) to silence this warning.",
];

// nvm: nvm_set_colors
pub fn setting_colors(colors: &str) -> String {
    format!("Setting colors to: {colors}")
}
pub const COLORS_NOT_SUPPORTED: &str =
    "WARNING: Colors may not display because they are not supported in this shell.";
pub const INVALID_COLOR_CODES: &str = "\\033[1;37mPlease pass in five \\033[1;31mvalid color codes\\033[1;37m. Choose from: rRgGbBcCyYmMkKeW\\033[0m";

// nvm's own source: trailing slashes on NVM_DIR.
pub const NVM_DIR_TRAILING_SLASH: &str = "Warning: $NVM_DIR should not have trailing slashes";

// string-audit: skip, printed by the `ls` program nvm_normalize_lts runs.
pub fn ls_missing(dir: &str) -> String {
    if cfg!(target_os = "macos") {
        format!("ls: {dir}: No such file or directory")
    } else {
        format!("ls: cannot access '{dir}': No such file or directory")
    }
}

// string-audit: skip, printed by the `tee` program nvm_make_alias runs.
pub fn tee_failed(path: &str, error: &std::io::Error) -> String {
    format!("tee: {path}: {}", strerror(error))
}

/// The C library's text for an error, without Rust's "(os error N)".
pub fn strerror(error: &std::io::Error) -> String {
    match error.raw_os_error() {
        // SAFETY: strerror returns a pointer to a static, NUL-terminated string.
        Some(code) => unsafe {
            std::ffi::CStr::from_ptr(libc::strerror(code))
                .to_string_lossy()
                .into_owned()
        },
        None => error.to_string(),
    }
}
