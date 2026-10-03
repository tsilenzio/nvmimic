//! `nvm alias` and `nvm unalias`.

use crate::context::Nvm;
use crate::nvm::alias::{self, AliasStyle};
use crate::nvm::local;
use crate::text::messages;

/// nvm: nvm:alias
pub fn run(nvm: &mut Nvm, args: &[String]) -> i32 {
    let alias_dir = nvm.backend.alias_dir(&nvm.vars);
    let current = local::ls_current(nvm);
    nvm.backend.make_dirs(&format!("{alias_dir}/lts"));

    let mut name: Option<String> = None;
    let mut target: Option<String> = None;
    let mut no_colors = false;
    for arg in args {
        match arg.as_str() {
            "--" => {}
            "--no-colors" => no_colors = true,
            other if other.starts_with("--") => {
                nvm.io.err(&messages::unsupported_option(other));
                return 55;
            }
            other => {
                if name.is_none() {
                    name = Some(other.to_string());
                } else if target.is_none() {
                    target = Some(other.to_string());
                }
            }
        }
    }

    let saved = nvm.local("NVM_NO_COLORS");
    if no_colors {
        nvm.vars.set_quiet("NVM_NO_COLORS", "--no-colors");
    }
    let status = run_parsed(nvm, name, target, &current);
    nvm.restore("NVM_NO_COLORS", saved);
    status
}

fn run_parsed(nvm: &mut Nvm, name: Option<String>, target: Option<String>, current: &str) -> i32 {
    // nvm marks "not given" with the literal `--`, so `nvm alias -- --` and
    // friends behave as if nothing was given.
    let name = name.unwrap_or_else(|| "--".to_string());
    let target = target.unwrap_or_else(|| "--".to_string());

    if target.is_empty() {
        super::nvm(nvm, &["unalias".to_string(), name])
    } else if name.contains('#') {
        nvm.io.err(messages::ALIAS_COMMENT);
        1
    } else if target != "--" {
        if name.contains('/') {
            nvm.io.err(messages::ALIAS_SUBDIRECTORY);
            return 1;
        }
        let version = local::version(nvm, &target).0;
        if version == "N/A" {
            nvm.io.err(&messages::version_does_not_exist(&target));
        }
        alias::make_alias(nvm, &name, &target);
        alias::print_formatted_alias(
            nvm,
            &name,
            &target,
            Some(&version),
            AliasStyle::default(),
            current,
        );
        let no_colors = nvm.vars.get("NVM_NO_COLORS").to_string();
        nvm.prefix_assignments(&[
            ("NVM_NO_COLORS", &no_colors),
            ("NVM_CURRENT", current),
            ("DEFAULT", "false"),
        ]);
        0
    } else {
        let prefix = if name == "--" { "" } else { name.as_str() };
        alias::list_aliases(nvm, prefix)
    }
}

/// nvm: nvm:unalias
pub fn unalias(nvm: &mut Nvm, args: &[String]) -> i32 {
    let alias_dir = nvm.backend.alias_dir(&nvm.vars);
    nvm.backend.make_dirs(&alias_dir);
    if args.len() != 1 {
        for line in messages::UNALIAS_USAGE {
            nvm.io.err(line);
        }
        return 127;
    }
    let name = &args[0];
    if name.contains('/') {
        nvm.io.err(messages::ALIAS_SUBDIRECTORY);
        return 1;
    }
    if !nvm.backend.alias_exists(&nvm.vars, name) {
        if matches!(
            name.as_str(),
            "stable" | "unstable" | "iojs" | "node" | "system"
        ) {
            nvm.io.err(&messages::builtin_alias(name));
            return 1;
        }
        nvm.io.err(&messages::alias_does_not_exist(name));
        return 0;
    }
    let original = alias::alias(nvm, name).0;
    nvm.backend.remove_alias(&nvm.vars, name);
    nvm.io.echo(&messages::deleted_alias(name, &original));
    0
}
