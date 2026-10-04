//! `nvm install-latest-npm` and `nvm reinstall-packages`.

use crate::context::Nvm;
use crate::nvm::{local, npm, tools};
use crate::text::install as text;

/// nvm: nvm:install-latest-npm
pub fn install_latest_npm(nvm: &mut Nvm, args: &[String]) -> i32 {
    if !args.is_empty() {
        for line in text::INSTALL_LATEST_NPM_USAGE {
            nvm.io.err(line);
        }
        return 127;
    }
    npm::install_latest_npm(nvm)
}

/// Installs, in the active version, the global packages another installed
/// version has, and links again the ones that were linked. `command` is the
/// name it was called by, `reinstall-packages` or `copy-packages`.
///
/// nvm: nvm:reinstall-packages
pub fn reinstall_packages(nvm: &mut Nvm, command: &str, args: &[String]) -> i32 {
    let [provided] = args else {
        nvm.io.err(&text::reinstall_usage(command));
        nvm.io.err(text::RUN_HELP);
        return 127;
    };

    let current = local::ls_current(nvm);
    if *provided == current || local::version(nvm, provided).0 == local::ls_current(nvm) {
        nvm.io.err(text::REINSTALL_FROM_CURRENT);
        return 2;
    }

    // nvm declares `local VERSION` here a second time, after doing so at
    // the top of `nvm()`. zsh prints a variable that is declared again.
    if nvm.dialect.is_zsh() {
        nvm.io.echo("VERSION=''");
    }
    let version = if provided == "system" {
        if !local::has_system_node(nvm) && !local::has_system_iojs(nvm) {
            nvm.io.err(text::NO_SYSTEM_NODE_OR_IOJS);
            return 3;
        }
        "system".to_string()
    } else {
        local::version(nvm, provided).0
    };

    let listing = npm::npm_global_modules(nvm, &version);
    let installs = listing.split(" //// ").next().unwrap_or("");
    let links = listing.rsplit(" //// ").next().unwrap_or("");

    let mut status = 0;
    nvm.io.echo(&text::reinstalling_from(&version));
    if !installs.is_empty() {
        npm::xargs_npm_install(nvm, installs);
    } else {
        nvm.io.echo(text::NO_GLOBAL_PACKAGES);
    }

    nvm.io.echo(&text::linking_from(&version));
    if !links.is_empty() {
        for link in links.lines().filter(|link| !link.is_empty()) {
            let dir = if link.starts_with('/') {
                link.to_string()
            } else {
                let root = tools::run(nvm, "npm", &["root", "-g"])
                    .map(|ran| ran.text())
                    .unwrap_or_default();
                format!("{root}/../{link}")
            };
            status = npm::link_in(nvm, &dir);
        }
    } else {
        nvm.io.echo(text::NO_LINKED_PACKAGES);
    }
    status
}
