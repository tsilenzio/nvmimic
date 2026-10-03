//! npm, as `nvm use` sees it: its version and the npmrc settings nvm refuses.

use super::local;
use super::paths::{self, tree_contains_path};
use super::semver;
use crate::context::Nvm;
use crate::text::messages;
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
fn npmrc_bad_news_bears(path: &str) -> bool {
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
