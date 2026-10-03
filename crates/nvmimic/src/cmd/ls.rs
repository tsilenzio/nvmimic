//! `nvm ls`: installed versions, then the aliases.

use crate::context::Nvm;
use crate::nvm::{local, output};
use crate::text::messages;

/// nvm: nvm:ls
pub fn run(nvm: &mut Nvm, args: &[String]) -> i32 {
    let mut pattern: Option<String> = None;
    let mut no_colors = false;
    let mut no_alias = false;
    for arg in args {
        match arg.as_str() {
            "--" => {}
            "--no-colors" => no_colors = true,
            "--no-alias" => no_alias = true,
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
    let pattern = pattern.unwrap_or_default();
    if !pattern.is_empty() && no_alias {
        nvm.io.err(messages::NO_ALIAS_WITH_PATTERN);
        return 55;
    }

    let saved = nvm.local("NVM_NO_COLORS");
    if no_colors {
        nvm.vars.set_quiet("NVM_NO_COLORS", "--no-colors");
    }

    let (listing, status) = local::ls(nvm, &pattern);
    output::print_versions(nvm, &listing);
    let no_colors_value = nvm.vars.get("NVM_NO_COLORS").to_string();
    nvm.prefix_assignments(&[("NVM_NO_COLORS", &no_colors_value)]);
    if !no_alias && pattern.is_empty() {
        let args: Vec<String> = if no_colors {
            vec!["alias".into(), "--no-colors".into()]
        } else {
            vec!["alias".into()]
        };
        super::nvm(nvm, &args);
    }

    nvm.restore("NVM_NO_COLORS", saved);
    status
}
