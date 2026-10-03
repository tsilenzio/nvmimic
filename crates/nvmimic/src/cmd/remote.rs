//! `nvm ls-remote` and `nvm version-remote`.

use crate::context::Nvm;
use crate::nvm::{output, remote};
use crate::text::messages;

/// nvm: nvm:ls-remote
pub fn ls_remote(nvm: &mut Nvm, args: &[String]) -> i32 {
    let mut lts: Option<String> = None;
    let mut pattern: Option<String> = None;
    let mut no_colors = false;
    for arg in args {
        match arg.as_str() {
            "--" => {}
            "--lts" => lts = Some("*".to_string()),
            "--no-colors" => no_colors = true,
            other if other.starts_with("--lts=") => lts = Some(other["--lts=".len()..].to_string()),
            other if other.starts_with("--") => {
                nvm.io.err(&messages::unsupported_option(other));
                return 55;
            }
            other => {
                if pattern.as_deref().is_none_or(str::is_empty) {
                    pattern = Some(other.to_string());
                    if lts.as_deref().is_none_or(str::is_empty) {
                        if other == "lts/*" {
                            lts = Some("*".to_string());
                            pattern = Some(String::new());
                        } else if let Some(name) = other.strip_prefix("lts/") {
                            lts = Some(name.to_string());
                            pattern = Some(String::new());
                        }
                    }
                }
            }
        }
    }

    let saved_colors = nvm.local("NVM_NO_COLORS");
    if no_colors {
        nvm.vars.set_quiet("NVM_NO_COLORS", "--no-colors");
    }
    let saved_lts = nvm.local("NVM_LTS");
    if let Some(lts) = &lts {
        nvm.vars.set_quiet("NVM_LTS", lts);
    }
    let (listing, status) = remote::remote_versions(nvm, pattern.as_deref().unwrap_or(""));
    nvm.restore("NVM_LTS", saved_lts);

    let no_colors_value = nvm.vars.get("NVM_NO_COLORS").to_string();
    let status = if !listing.is_empty() {
        output::print_versions(nvm, &listing);
        status
    } else {
        output::print_versions(nvm, "N/A");
        3
    };
    nvm.prefix_assignments(&[("NVM_NO_COLORS", &no_colors_value)]);
    nvm.restore("NVM_NO_COLORS", saved_colors);
    status
}

/// nvm: nvm:version-remote
pub fn version_remote(nvm: &mut Nvm, args: &[String]) -> i32 {
    let mut lts: Option<String> = None;
    let mut pattern: Option<String> = None;
    for arg in args {
        match arg.as_str() {
            "--" => {}
            "--lts" => lts = Some("*".to_string()),
            other if other.starts_with("--lts=") => lts = Some(other["--lts=".len()..].to_string()),
            other if other.starts_with("--") => {
                nvm.io.err(&messages::unsupported_option(other));
                return 55;
            }
            other => {
                if pattern.as_deref().is_none_or(str::is_empty) {
                    pattern = Some(other.to_string());
                }
            }
        }
    }
    match pattern.as_deref() {
        Some("lts/*") => {
            lts = Some("*".to_string());
            pattern = None;
        }
        Some(p) if p.starts_with("lts/") => {
            lts = Some(p["lts/".len()..].to_string());
            pattern = None;
        }
        _ => {}
    }
    let pattern = match pattern.as_deref() {
        None | Some("") => "node".to_string(),
        Some(p) => p.to_string(),
    };
    let saved_only = nvm.local("NVM_VERSION_ONLY");
    let saved_lts = nvm.local("NVM_LTS");
    nvm.vars.set_quiet("NVM_VERSION_ONLY", "true");
    if let Some(lts) = &lts {
        nvm.vars.set_quiet("NVM_LTS", lts);
    }
    let lts = nvm.vars.get("NVM_LTS").to_string();
    let (text, status) = remote::remote_version(nvm, &pattern);
    nvm.io.echo(&text);
    nvm.restore("NVM_LTS", saved_lts);
    nvm.restore("NVM_VERSION_ONLY", saved_only);
    nvm.prefix_assignments(&[("NVM_VERSION_ONLY", "true"), ("NVM_LTS", &lts)]);
    status
}
