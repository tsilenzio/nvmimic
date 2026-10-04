//! npm, as `nvm use` sees it: its version and the npmrc settings nvm refuses.

use super::local;
use super::paths::{self, tree_contains_path};
use super::semver;
use crate::context::Nvm;
use crate::text::install as text;
use crate::text::messages;
use crate::text::npm_ladder as ladder;
use std::path::Path;

/// The version of the npm package a path leads to, when it can be trusted
/// to be what `npm --version` prints: the link resolves into a package named
/// `npm`, and that npm supports the active node. `None` for anything else,
/// such as Corepack's shim or a wrapper script.
fn bundled_npm_version(npm: &str, node_version: &str) -> Option<String> {
    let resolved = std::fs::canonicalize(npm).ok()?;
    let manifest = resolved
        .ancestors()
        .skip(1)
        .map(|dir| dir.join("package.json"))
        .find(|path| path.is_file())?;
    let package: serde_json::Value = serde_json::from_slice(&std::fs::read(manifest).ok()?).ok()?;
    if package.get("name")?.as_str()? != "npm" {
        return None;
    }
    let engines = package.get("engines")?.get("node")?.as_str()?;
    if !semver::satisfies(node_version, engines) {
        return None;
    }
    Some(package.get("version")?.as_str()?.to_string())
}

/// What `npm --version` prints for the `npm` the shell would run, found by
/// scanning the caller's `PATH`. The version comes from npm's `package.json`
/// when [`bundled_npm_version`] can vouch for it. Otherwise npm is run, as
/// nvm runs it, and printing nothing means there is no version.
pub fn npm_version(nvm: &Nvm) -> Option<String> {
    let npm = paths::which(nvm.vars.get("PATH"), "npm")?;
    let node = local::which_node(nvm, "node")
        .and_then(|node| local::binary_version(&node))
        .unwrap_or_default();
    if let Some(version) = bundled_npm_version(&npm, &node) {
        return Some(version);
    }
    let mut command = std::process::Command::new(&npm);
    command
        .arg("--version")
        .env_clear()
        .envs(nvm.vars.exported())
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    if !nvm.vars.get("PWD").is_empty() {
        command.current_dir(nvm.vars.get("PWD"));
    }
    let output = command.output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    Some(text.trim_end_matches('\n').to_string())
}

/// ` (npm vX.Y.Z)` for the `npm` on `PATH`, or nothing.
///
/// nvm: nvm_print_npm_version
pub fn print_npm_version(nvm: &Nvm) -> String {
    npm_version(nvm)
        .filter(|version| !version.is_empty())
        .map(|version| messages::npm_suffix(&version))
        .unwrap_or_default()
}

/// `grep -Ee '^(prefix|globalconfig) *='` over an npmrc file.
///
/// nvm: nvm_npmrc_bad_news_bears
pub fn npmrc_bad_news_bears(path: &str) -> bool {
    if path.is_empty() || !Path::new(path).is_file() {
        return false;
    }
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    String::from_utf8_lossy(&bytes).lines().any(|line| {
        ["prefix", "globalconfig"].iter().any(|key| {
            line.strip_prefix(key)
                .is_some_and(|rest| rest.trim_start_matches(' ').starts_with('='))
        })
    })
}

/// Refuses an npm configuration that would put global packages outside
/// nvm's install. Returns nvm's status: 3 for `$PREFIX`, 4 for an
/// `npm_config_prefix` variable, 10 for an npmrc setting.
///
/// nvm: nvm_die_on_prefix
pub fn die_on_prefix(nvm: &mut Nvm, delete_prefix: bool, command: &str, version_dir: &str) -> i32 {
    if command.is_empty() || version_dir.is_empty() {
        nvm.io.err("Second argument \"nvm command\", and third argument \"nvm version dir\", must both be nonempty");
        return 2;
    }

    let prefix = nvm.vars.get("PREFIX").to_string();
    if !prefix.is_empty() {
        let node = local::which_node(nvm, "node")
            .and_then(|node| local::binary_version(&node))
            .unwrap_or_default();
        let node_dir = paths::version_path(nvm, &node).unwrap_or_default();
        if node_dir != prefix {
            crate::cmd::deactivate::deactivate_quietly(nvm);
            nvm.io.err(&messages::prefix_env(&prefix));
            nvm.io.err(messages::UNSET_PREFIX);
            return 3;
        }
    }

    let nvm_dir = nvm.vars.get("NVM_DIR").to_string();
    let env_name = nvm
        .vars
        .exported_names()
        .into_iter()
        .find(|name| name.eq_ignore_ascii_case("NPM_CONFIG_PREFIX"));
    if let Some(name) = env_name {
        let value = nvm.vars.get(&name).to_string();
        if !value.is_empty() && tree_contains_path(&nvm_dir, &value) != Some(true) {
            crate::cmd::deactivate::deactivate_quietly(nvm);
            nvm.io.err(&messages::config_prefix_env(&name, &value));
            nvm.io.err(&messages::unset_env(&name));
            return 4;
        }
    }

    let home = nvm.vars.get("HOME").to_string();
    let pwd = nvm.vars.get("PWD").to_string();
    let checks: [(String, Describe, Npmrc); 4] = [
        (
            format!("{version_dir}/lib/node_modules/npm/npmrc"),
            messages::builtin_npmrc,
            Npmrc::UserConfig,
        ),
        (
            format!("{version_dir}/etc/npmrc"),
            messages::global_npmrc,
            Npmrc::Global,
        ),
        (
            format!("{home}/.npmrc"),
            messages::user_npmrc,
            Npmrc::UserConfig,
        ),
        (
            format!("{}/.npmrc", paths::find_project_dir(&pwd)),
            messages::project_npmrc,
            Npmrc::Project,
        ),
    ];
    for (path, describe, kind) in checks {
        if npmrc_bad_news_bears(&path) {
            if delete_prefix {
                delete_npmrc_prefix(nvm, &path, kind);
            } else {
                let shown = paths::sanitize(&path, &nvm_dir, &home);
                nvm.io.err(&describe(&shown));
                nvm.io.err(messages::NPMRC_INCOMPATIBLE);
                nvm.io.err(&messages::run_to_unset(command));
                return 10;
            }
        }
    }
    0
}

/// Names an npmrc file in nvm's error message.
type Describe = fn(&str) -> String;

/// How nvm asks npm to edit each npmrc file.
#[derive(Clone, Copy)]
enum Npmrc {
    UserConfig,
    Global,
    Project,
}

/// `nvm use --delete-prefix`: runs npm to remove `prefix` and `globalconfig`
/// from one npmrc file, as nvm does. Only this rare path starts npm.
fn delete_npmrc_prefix(nvm: &mut Nvm, path: &str, kind: Npmrc) {
    let Some(npm) = paths::which(nvm.vars.get("PATH"), "npm") else {
        return;
    };
    for key in ["prefix", "globalconfig"] {
        let mut command = std::process::Command::new(&npm);
        command.arg("config");
        if matches!(kind, Npmrc::Global) {
            command.arg("--global");
        }
        command.args(["--loglevel=warn", "delete", key]);
        if matches!(kind, Npmrc::UserConfig) {
            command.arg(format!("--userconfig={path}"));
        }
        command.env_clear().envs(nvm.vars.exported());
        if let Ok(output) = command.output() {
            nvm.io.out_raw(&String::from_utf8_lossy(&output.stdout));
            nvm.io.err_raw(&String::from_utf8_lossy(&output.stderr));
        }
    }
}

/// What `npm --version` prints, by running the `npm` on `PATH`. Empty when
/// there is none or it prints nothing.
fn run_npm_version(nvm: &mut Nvm) -> String {
    super::tools::run_quiet(nvm, "npm", &["--version"])
        .map(|ran| ran.text())
        .unwrap_or_default()
}

/// Runs `npm <args>` in the foreground.
fn run_npm(nvm: &mut Nvm, args: &[&str]) -> i32 {
    match super::tools::command(nvm, "npm") {
        Some(mut command) => {
            command.args(args);
            super::tools::run_through(nvm, command)
        }
        None => 127,
    }
}

/// `(nvm_cd <dir> && npm link)`: links the package in a directory into the
/// active version's global packages. Returns 1 when the directory cannot
/// be entered.
pub fn link_in(nvm: &mut Nvm, dir: &str) -> i32 {
    if !Path::new(dir).is_dir() {
        return 1;
    }
    match super::tools::command(nvm, "npm") {
        Some(mut command) => {
            command.arg("link").current_dir(dir).env("PWD", dir);
            super::tools::run_through(nvm, command)
        }
        None => 127,
    }
}

/// `nvm_echo "<words>" | command xargs npm install -g --quiet`. The real
/// `xargs` runs, since what it says when npm is missing differs from system
/// to system.
pub fn xargs_npm_install(nvm: &mut Nvm, words: &str) -> i32 {
    match super::tools::command(nvm, "xargs") {
        Some(mut command) => {
            command.args(["npm", "install", "-g", "--quiet"]);
            super::tools::run_fed(nvm, command, format!("{words}\n").as_bytes())
        }
        None => 127,
    }
}

/// Installs npm with its own installer where a Node release came without
/// one. Only Node releases far older than any current one do.
///
/// nvm: nvm_install_npm_if_needed
pub fn install_npm_if_needed(nvm: &mut Nvm) -> i32 {
    let version = local::ls_current(nvm);
    if paths::on_path(nvm.vars.get("PATH"), "npm") {
        return 0;
    }
    nvm.io.echo(text::INSTALLING_NPM);
    let installer = |nvm: &mut Nvm, pinned: Option<&str>| -> i32 {
        let script = super::remote::download_capture(
            nvm,
            &["-L", "https://npmjs.org/install.sh", "-o", "-"],
        );
        match super::tools::command(nvm, "sh") {
            Some(mut command) => {
                command.env("clean", "yes");
                if let Some(pinned) = pinned {
                    command.env("npm_install", pinned);
                }
                super::tools::run_fed(nvm, command, &script)
            }
            None => 127,
        }
    };
    if super::version::greater("0.2.0", &version) {
        nvm.io.err(text::NPM_NEEDS_NODE);
        0
    } else if super::version::gte(&version, "0.2.0") {
        if super::version::greater("0.2.3", &version) {
            nvm.io.err(text::NPM_NEEDS_NODE);
            0
        } else {
            installer(nvm, Some("0.2.19"))
        }
    } else {
        installer(nvm, None)
    }
}

/// The packages listed in `$NVM_DIR/default-packages`, as nvm's awk script
/// joins them, and its status: 1 when a line holds more than one word.
///
/// The join has a quirk that is kept. A space goes before every package
/// that is not on the file's first line, so a file that opens with a
/// comment gives a list that starts with a space.
///
/// nvm: nvm_get_default_packages
pub fn get_default_packages(nvm: &mut Nvm) -> (String, i32) {
    let file = format!("{}/default-packages", nvm.vars.get("NVM_DIR"));
    if !Path::new(&file).is_file() {
        return (String::new(), 0);
    }
    let Ok(bytes) = std::fs::read(&file) else {
        return (String::new(), 2);
    };
    let content = String::from_utf8_lossy(&bytes);
    let blank = |c: char| c == ' ' || c == '\t';
    let mut joined = String::new();
    for (index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start_matches(blank);
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if line.contains(blank) {
            nvm.io.err(&text::one_package_per_line(&file));
            return (joined, 1);
        }
        if index > 0 {
            joined.push(' ');
        }
        joined.push_str(line);
    }
    (joined, 0)
}

/// What [`install_default_packages`] left behind.
pub struct DefaultPackages {
    pub status: i32,
    /// nvm assigns the status of reading the list to `EXIT_CODE` without
    /// declaring it, so it lands in the caller's variable of that name.
    pub exit_code: i32,
}

/// Installs the packages from `$NVM_DIR/default-packages` with npm.
///
/// nvm: nvm_install_default_packages
pub fn install_default_packages(nvm: &mut Nvm) -> DefaultPackages {
    let (listed, exit_code) = nvm.subshell(|nvm| get_default_packages(nvm));
    let packages = listed.trim_end_matches('\n');
    if exit_code != 0 || packages.is_empty() {
        return DefaultPackages {
            status: exit_code,
            exit_code,
        };
    }
    let nvm_dir = nvm.vars.get("NVM_DIR").to_string();
    nvm.io.echo(&text::installing_default_packages(&nvm_dir));
    nvm.io.echo(&text::npm_install_quiet(packages));
    let mut status = 0;
    if xargs_npm_install(nvm, packages) != 0 {
        nvm.io.err(text::DEFAULT_PACKAGES_FAILED);
        status = 1;
    }
    DefaultPackages { status, exit_code }
}

/// Upgrades npm to the newest release that still runs on the active Node.
/// Every `install -g` goes through `npm` on `PATH`. With `NVM_DEBUG=1` the
/// commands are printed instead of run.
///
/// nvm: nvm_install_latest_npm
pub fn install_latest_npm(nvm: &mut Nvm) -> i32 {
    use super::version::{greater, gte, strip_iojs_prefix};
    nvm.io.echo(text::UPGRADING_NPM);
    let current = local::ls_current(nvm);
    let mut node = strip_iojs_prefix(&current).to_string();
    let npm = run_npm_version(nvm);

    if node == "system" {
        node = super::tools::run(nvm, "node", &["--version"])
            .map(|ran| ran.text())
            .unwrap_or_default();
    } else if node == "none" {
        nvm.io.echo(&text::detected_versions(&node, &npm));
        node = String::new();
    }
    if node.is_empty() {
        nvm.io.err(text::NO_NODE_VERSION);
        return 1;
    }
    if npm.is_empty() {
        nvm.io.err(text::NO_NPM_VERSION);
        return 2;
    }

    let debug = nvm.vars.get("NVM_DEBUG") == "1";
    if debug {
        nvm.io.echo(&text::detected_versions(&node, &npm));
    }
    let install = |nvm: &mut Nvm, package: &str| {
        if debug {
            nvm.io.echo(&format!("npm install -g {package}"));
        } else {
            run_npm(nvm, &["install", "-g", package]);
        }
    };
    let step = |nvm: &mut Nvm, (note, package): (&str, &str)| {
        nvm.io.echo(note);
        install(nvm, package);
    };

    let node = node.as_str();
    let at_least = |version: &str| gte(node, version);
    let is_0_6 = at_least("0.6.0") && greater("0.7.0", node);
    let is_0_9 = at_least("0.9.0") && greater("0.10.0", node);

    if is_0_6 {
        step(nvm, (ladder::NODE_0_6, "npm@1.3"));
    } else if !is_0_9 {
        // node 0.9 breaks here, for some reason
        if gte(&npm, "1.0.0") && greater("2.0.0", &npm) {
            step(nvm, (ladder::NPM_1, "npm@1.4.28"));
        } else if gte(&npm, "2.0.0") && greater("3.0.0", &npm) {
            step(nvm, (ladder::NPM_2, "npm@2"));
        }
    }

    if is_0_9 || is_0_6 {
        nvm.io.echo(ladder::STUCK);
    } else if greater("1.1.0", node) {
        step(nvm, (ladder::BELOW_1_1, "npm@4.5"));
    } else if greater("4.0.0", node) {
        step(nvm, (ladder::BELOW_4, "npm@4"));
    } else {
        // Each line of nvm's ladder only holds when the one before it does.
        let is_4_4_or_below = greater("4.5.0", node);
        let is_5_or_above = !is_4_4_or_below && at_least("5.0.0");
        let is_6_or_above = is_5_or_above && at_least("6.0.0");
        let is_6_2_or_above = is_6_or_above && at_least("6.2.0");
        let is_9_or_above = is_6_2_or_above && at_least("9.0.0");
        let is_9_3_or_above = is_9_or_above && at_least("9.3.0");
        let is_10_or_above = is_9_3_or_above && at_least("10.0.0");
        let is_12_lts_or_above = is_10_or_above && at_least("12.13.0");
        let is_13_or_above = is_12_lts_or_above && at_least("13.0.0");
        let is_14_lts_or_above = is_13_or_above && at_least("14.15.0");
        let is_14_17_or_above = is_14_lts_or_above && at_least("14.17.0");
        let is_15_or_above = is_14_lts_or_above && at_least("15.0.0");
        let is_16_or_above = is_15_or_above && at_least("16.0.0");
        let is_16_lts_or_above = is_16_or_above && at_least("16.13.0");
        let is_17_or_above = is_16_lts_or_above && at_least("17.0.0");
        let is_18_or_above = is_17_or_above && at_least("18.0.0");
        let is_18_17_or_above = is_18_or_above && at_least("18.17.0");
        let is_19_or_above = is_18_17_or_above && at_least("19.0.0");
        let is_20_5_or_above = is_19_or_above && at_least("20.5.0");
        let is_20_17_or_above = is_20_5_or_above && at_least("20.17.0");
        let is_21_or_above = is_20_17_or_above && at_least("21.0.0");
        let is_22_9_or_above = is_21_or_above && at_least("22.9.0");

        if is_4_4_or_below || (is_5_or_above && greater("5.10.0", node)) {
            step(nvm, (ladder::NPM_5_3, "npm@5.3"));
        } else if !is_4_4_or_below && greater("4.7.0", node) {
            step(nvm, (ladder::NPM_5_4_1, "npm@5.4.1"));
        } else if !is_6_or_above {
            step(nvm, (ladder::NPM_5, "npm@5"));
        } else if (is_6_or_above && !is_6_2_or_above) || (is_9_or_above && !is_9_3_or_above) {
            step(nvm, (ladder::NPM_6_9, "npm@6.9"));
        } else if !is_10_or_above {
            if greater("4.4.4", &npm) {
                step(nvm, (ladder::NPM_4_FIRST, "npm@4"));
            }
            step(nvm, (ladder::NPM_6, "npm@6"));
        } else if !is_12_lts_or_above
            || (is_13_or_above && !is_14_lts_or_above)
            || (is_15_or_above && !is_16_or_above)
        {
            step(nvm, (ladder::NPM_7, "npm@7"));
        } else if (is_12_lts_or_above && !is_13_or_above)
            || (is_14_lts_or_above && !is_14_17_or_above)
            || (is_16_or_above && !is_16_lts_or_above)
            || (is_17_or_above && !is_18_or_above)
        {
            step(nvm, (ladder::NPM_8_6, "npm@8.6"));
        } else if !is_18_17_or_above || (is_19_or_above && !is_20_5_or_above) {
            step(nvm, (ladder::NPM_9, "npm@9"));
        } else if !is_20_17_or_above || (is_21_or_above && !is_22_9_or_above) {
            step(nvm, (ladder::NPM_10, "npm@10"));
        } else {
            step(nvm, (ladder::NPM_LATEST, "npm"));
        }
    }
    let upgraded = run_npm_version(nvm);
    nvm.io.echo(&text::npm_upgraded_to(&upgraded));
    0
}

/// The global packages of one installed version, as nvm hands them on:
/// `<installed packages> //// <linked package paths>`.
///
/// nvm: nvm_npm_global_modules
pub fn npm_global_modules(nvm: &mut Nvm, version: &str) -> String {
    // nvm switches to the version inside `$( ... )`, so the switch is gone
    // again afterwards.
    let listing = nvm.subshell(|nvm| {
        let used =
            nvm.quiet_out(|nvm| crate::cmd::nvm(nvm, &["use".to_string(), version.to_string()]));
        if used != 0 {
            return String::new();
        }
        super::tools::run_quiet(nvm, "npm", &["list", "-g", "--depth=0"])
            .map(|ran| String::from_utf8_lossy(&ran.stdout).into_owned())
            .unwrap_or_default()
    });
    let lines: Vec<&str> = listing
        .lines()
        .skip(1)
        .filter(|line| !line.contains("UNMET PEER DEPENDENCY"))
        .collect();

    let package = super::posix_re::bre(r"^.* \(.*@[^ ]*\).*");
    let installs: Vec<String> = lines
        .iter()
        // `\(empty\)` in sed is a group, so this drops every line holding
        // the word, parentheses or not.
        .filter(|line| !line.contains(" -> ") && !line.contains("empty"))
        .map(|line| match &package {
            Some(re) => super::posix_re::sed_first(line, re, r"\1"),
            None => line.to_string(),
        })
        .filter(|line| !line.starts_with("npm@") && !line.starts_with("corepack@"))
        .collect();
    // `xargs` with no command joins every word with one space.
    let installs = installs
        .iter()
        .flat_map(|line| line.split_whitespace())
        .collect::<Vec<_>>()
        .join(" ");

    let links: Vec<&str> = lines
        .iter()
        .filter_map(|line| line.rfind(" -> ").map(|at| &line[at + 4..]))
        .collect();
    format!("{installs} //// {}", links.join("\n"))
}
