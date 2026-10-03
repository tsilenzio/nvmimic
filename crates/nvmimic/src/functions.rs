//! nvm's helper functions as callable entry points. Sourcing `nvm.sh` defines
//! every one of them in the shell, and other scripts call them, so each name
//! leads somewhere: a port where one exists, real nvm otherwise.

use crate::cmd::{self, fallback};
use crate::context::Nvm;
use crate::nvm::{alias, colors, local, npm, nvmrc, output, paths, platform, remote, version};

/// Runs helper `name`. `fd3` receives what nvm writes to file descriptor 3.
pub fn call(nvm: &mut Nvm, name: &str, args: &[String], fd3: &mut dyn FnMut(&str)) -> i32 {
    let arg = |i: usize| args.get(i).map_or("", String::as_str);
    let print = |nvm: &mut Nvm, (text, status): (String, i32)| {
        if !text.is_empty() {
            nvm.io.echo(&text);
        }
        status
    };
    match name {
        "nvm" => cmd::nvm(nvm, args),
        "nvm_auto" => auto(nvm, arg(0)),
        "nvm_rc_version" => {
            let silent = nvm.vars.get("NVM_SILENT") == "1";
            let (version, status) = nvmrc::rc_version(nvm, silent);
            if let Some(version) = version {
                fd3(&format!("{version}\n"));
            }
            status
        }
        "nvm_ls" => {
            let result = local::ls(nvm, arg(0));
            print(nvm, result)
        }
        "nvm_version" => {
            let (text, status) = local::version(nvm, arg(0));
            nvm.io.echo(&text);
            status
        }
        "nvm_match_version" => {
            let (text, status) = local::match_version(nvm, arg(0));
            nvm.io.echo(&text);
            status
        }
        "nvm_ls_current" => {
            nvm.io.echo(&local::ls_current(nvm));
            0
        }
        "nvm_alias" => {
            let result = alias::alias(nvm, arg(0));
            print(nvm, result)
        }
        "nvm_resolve_alias" => {
            let result = alias::resolve_alias(nvm, arg(0));
            print(nvm, result)
        }
        "nvm_resolve_local_alias" => {
            let result = alias::resolve_local_alias(nvm, arg(0));
            print(nvm, result)
        }
        "nvm_list_aliases" => alias::list_aliases(nvm, arg(0)),
        "nvm_ensure_version_installed" => {
            local::ensure_version_installed(nvm, arg(0), arg(1) == "1")
        }
        "nvm_is_version_installed" => (!local::is_version_installed(nvm, arg(0))) as i32,
        "nvm_has_system_node" => (!local::has_system_node(nvm)) as i32,
        "nvm_has_system_iojs" => (!local::has_system_iojs(nvm)) as i32,
        "nvm_has_colors" => (!colors::has_colors(nvm)) as i32,
        "nvm_version_path" => match paths::version_path(nvm, arg(0)) {
            Ok(path) => {
                nvm.io.echo(&path);
                0
            }
            Err(message) => {
                nvm.io.err(&message);
                3
            }
        },
        "nvm_is_valid_version" => (!version::is_valid(arg(0))) as i32,
        "nvm_version_greater" => version::greater_status(arg(0), arg(1)),
        "nvm_version_greater_than_or_equal_to" => version::gte_status(arg(0), arg(1)),
        "nvm_normalize_version" => {
            nvm.io.echo(&version::normalize(arg(0)));
            0
        }
        "nvm_num_version_groups" => {
            nvm.io.echo(&version::num_groups(arg(0)).to_string());
            0
        }
        "nvm_format_version" => {
            nvm.io.echo(&version::format(arg(0)));
            0
        }
        "nvm_ensure_version_prefix" => {
            nvm.io.echo(&version::ensure_prefix(arg(0)));
            0
        }
        "nvm_add_iojs_prefix" => {
            nvm.io.echo(&version::add_iojs_prefix(arg(0)));
            0
        }
        "nvm_validate_implicit_alias" => {
            if version::is_implicit_alias(arg(0)) {
                0
            } else {
                nvm.io.err(&crate::text::messages::only_implicit_aliases());
                1
            }
        }
        "nvm_strip_path" => {
            if nvm.vars.get("NVM_DIR").is_empty() {
                nvm.io.err("${NVM_DIR} not set!");
                return 1;
            }
            let stripped = paths::strip_path(arg(0), nvm.vars.get("NVM_DIR"), arg(1));
            nvm.io.out_raw(&stripped);
            0
        }
        "nvm_change_path" => {
            let changed = paths::change_path(arg(0), arg(1), arg(2), nvm.vars.get("NVM_DIR"));
            nvm.io.echo(&changed);
            0
        }
        "nvm_sanitize_path" => {
            let sanitized = paths::sanitize(arg(0), nvm.vars.get("NVM_DIR"), nvm.vars.get("HOME"));
            nvm.io.echo(&sanitized);
            0
        }
        "nvm_print_versions" => {
            output::print_versions(nvm, arg(0));
            0
        }
        "nvm_print_npm_version" => {
            let suffix = npm::print_npm_version(nvm);
            nvm.io.out_raw(&suffix);
            0
        }
        "nvm_write_nvmrc" => {
            let silent = nvm.vars.get("NVM_SILENT") == "1";
            nvmrc::write_nvmrc(nvm, arg(0), silent)
        }
        "nvm_process_nvmrc_content" => match nvmrc::process_content(nvm, arg(0)) {
            Some(version) => {
                nvm.io.echo(&version);
                0
            }
            None => 1,
        },
        "nvm_ls_remote" => {
            let result = remote::ls_remote(nvm, arg(0));
            print(nvm, result)
        }
        "nvm_ls_remote_iojs" => {
            let result = remote::ls_remote_iojs(nvm, arg(0));
            print(nvm, result)
        }
        "nvm_ls_remote_index_tab" => {
            if args.len() < 3 {
                nvm.io.err(crate::text::messages::NOT_ENOUGH_ARGUMENTS);
                return 5;
            }
            let result = remote::ls_remote_index_tab(nvm, arg(0), arg(1), Some(arg(2)));
            print(nvm, result)
        }
        "nvm_remote_versions" => {
            let result = remote::remote_versions(nvm, arg(0));
            print(nvm, result)
        }
        "nvm_remote_version" => {
            let (text, status) = remote::remote_version(nvm, arg(0));
            nvm.io.echo(&text);
            status
        }
        "nvm_get_mirror" => match remote::get_mirror(nvm, arg(0), arg(1)) {
            Ok(mirror) => {
                nvm.io.echo(&mirror);
                0
            }
            Err(status) => status,
        },
        "nvm_sanitize_auth_header" => {
            nvm.io.echo(&remote::sanitize_auth_header(arg(0)));
            0
        }
        "nvm_print_implicit_alias" => {
            let result = match arg(0) {
                "local" => alias::print_implicit_alias_local(nvm, arg(1)),
                "remote" => remote::print_implicit_alias_remote(nvm, arg(1)),
                _ => {
                    nvm.io.err(crate::text::messages::IMPLICIT_ALIAS_KIND);
                    return 1;
                }
            };
            print(nvm, result)
        }
        "nvm_get_os" => {
            let os = platform::get_os(nvm);
            nvm.io.echo(&os);
            0
        }
        "nvm_get_arch" => {
            let arch = platform::get_arch(nvm);
            nvm.io.echo(&arch);
            0
        }
        "nvm_get_minor_version" => match platform::get_minor_version(nvm, arg(0)) {
            Ok(minor) => {
                nvm.io.echo(&minor);
                0
            }
            Err(status) => status,
        },
        "nvm_is_merged_node_version" => platform::is_merged_node_version(arg(0)),
        "nvm_binary_available" => platform::binary_available(arg(0)),
        "nvm_iojs_version_has_solaris_binary" => platform::iojs_version_has_solaris_binary(arg(0)),
        "nvm_node_version_has_solaris_binary" => platform::node_version_has_solaris_binary(arg(0)),
        "nvm_has_solaris_binary" => platform::has_solaris_binary(arg(0)),
        "nvm_supports_xz" => platform::supports_xz(nvm, arg(0)),
        "__nvm_aliases" => {
            completion_aliases(nvm);
            0
        }
        _ => fallback::run_function(nvm, name, args),
    }
}

/// The alias files under `$NVM_DIR/alias`, relative to it, in the order
/// `find "${PWD}" -type f` walks them, one per line, for bash completion.
///
/// nvm: bash_completion __nvm_aliases
fn completion_aliases(nvm: &mut Nvm) {
    fn walk(dir: &std::path::Path, prefix: &str, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let name = entry.file_name().to_string_lossy().into_owned();
            if kind.is_file() {
                out.push(format!("{prefix}{name}"));
            } else if kind.is_dir() {
                walk(&entry.path(), &format!("{prefix}{name}/"), out);
            }
        }
    }
    let mut names = Vec::new();
    walk(
        std::path::Path::new(&nvm.backend.alias_dir(&nvm.vars)),
        "",
        &mut names,
    );
    if !names.is_empty() {
        nvm.io.echo(&names.join("\n"));
    }
}

/// What sourcing `nvm.sh` does after defining everything: switch to the
/// default version, or install it with `--install`.
///
/// nvm: nvm_auto
pub fn auto(nvm: &mut Nvm, mode: &str) -> i32 {
    match mode {
        "none" => 0,
        "use" => {
            let current = local::ls_current(nvm);
            let use_args = |version: Option<&str>| -> Vec<String> {
                let mut args = vec!["use".to_string(), "--silent".to_string()];
                if let Some(version) = version {
                    args.push(version.to_string());
                }
                args
            };
            if current == "none" || current == "system" {
                let version = nvm.quiet_err(|nvm| alias::resolve_local_alias(nvm, "default").0);
                if !version.is_empty() {
                    if version != "N/A" && version::is_valid(&version) {
                        nvm.quiet_out(|nvm| cmd::nvm(nvm, &use_args(Some(&version))))
                    } else {
                        0
                    }
                } else if nvm.quiet(|nvm| nvmrc::rc_version(nvm, false).1) == 0 {
                    nvm.quiet_out(|nvm| cmd::nvm(nvm, &use_args(None)))
                } else {
                    0
                }
            } else {
                nvm.quiet_out(|nvm| cmd::nvm(nvm, &use_args(Some(&current))))
            }
        }
        "install" => fallback::run_function(nvm, "nvm_auto", &["install".to_string()]),
        _ => {
            nvm.io.err("Invalid auto mode supplied.");
            1
        }
    }
}
