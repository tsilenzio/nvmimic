//! nvm's helper functions as callable entry points. Sourcing `nvm.sh` defines
//! every one of them in the shell, and other scripts call them, so each name
//! leads to its port.

use crate::cmd;
use crate::context::Nvm;
use crate::nvm::complaint;
use crate::nvm::{
    alias, artifact, checksum, colors, local, lock, npm, nvmrc, output, paths, platform, remote,
    version,
};

/// Runs helper `name`. `fd3` receives what nvm writes to file descriptor 3
/// and says whether that descriptor was open.
pub fn call(nvm: &mut Nvm, name: &str, args: &[String], fd3: &mut dyn FnMut(&str) -> bool) -> i32 {
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
            // nvm ends with `nvm_echo "${NVM_RC_VERSION}" >&3`. Called on
            // its own, nothing has opened descriptor 3, and the shell says so.
            match version {
                Some(version) if !fd3(&format!("{version}\n")) => {
                    let (line, status) =
                        complaint::bad_descriptor(nvm, &complaint::sites::RC_VERSION_FD3, 3);
                    nvm.io.err(&line);
                    status
                }
                _ => status,
            }
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
            let stripped = nvm.backend.strip_path(&nvm.vars, arg(0), arg(1));
            nvm.io.out_raw(&stripped);
            0
        }
        "nvm_change_path" => {
            let changed = nvm.backend.change_path(&nvm.vars, arg(0), arg(1), arg(2));
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
        "nvm_curl_libz_support" => (!remote::curl_libz_support(nvm)) as i32,
        "nvm_curl_version" => {
            let version = remote::curl_version(nvm);
            print(nvm, (version, 0))
        }
        "nvm_curl_use_compression" => remote::curl_use_compression_status(nvm),
        "nvm_download" => {
            let args: Vec<&str> = args.iter().map(String::as_str).collect();
            remote::download(nvm, &args)
        }
        "nvm_die_on_prefix" => {
            if !matches!(arg(0), "0" | "1") {
                nvm.io.err(crate::text::messages::DELETE_PREFIX_ARGUMENT);
                return 1;
            }
            npm::die_on_prefix(nvm, arg(0) == "1", arg(1), arg(2))
        }
        "nvm_npmrc_bad_news_bears" => (!npm::npmrc_bad_news_bears(arg(0))) as i32,
        "nvm_nvmrc_invalid_msg" => {
            nvmrc::invalid(nvm, arg(0));
            0
        }
        "nvm_get_colors" => match arg(0).parse::<usize>() {
            Ok(index @ 1..=6) if arg(0).len() == 1 => {
                let color = colors::get_color(nvm, index);
                nvm.io.echo(&color);
                0
            }
            _ => {
                nvm.io
                    .err(&crate::text::messages::invalid_color_index(arg(0)));
                1
            }
        },
        "nvm_print_color_code" => match colors::color_code(arg(0)) {
            Ok(Some(code)) => {
                nvm.io.echo(code);
                0
            }
            Ok(None) => 0,
            Err(message) => {
                nvm.io.err(&message);
                1
            }
        },
        "nvm_wrap_with_color_code" => {
            let wrapped = colors::wrap_with_color_code(nvm, arg(0), arg(1));
            nvm.io.echo(&wrapped);
            0
        }
        "nvm_print_color_legend" => {
            cmd::print_color_legend(nvm);
            0
        }
        "nvm_make_alias" => alias::make_alias(nvm, arg(0), arg(1)),
        "nvm_normalize_lts" => match alias::normalize_lts(nvm, arg(0)) {
            Ok(name) => {
                nvm.io.echo(&name);
                0
            }
            Err(status) => status,
        },
        "nvm_print_alias_file" => match std::fs::read(arg(0)) {
            Ok(bytes) => {
                let nul = alias::Nul::of(nvm);
                for line in alias::alias_file_lines(&String::from_utf8_lossy(&bytes), nul) {
                    nvm.io.echo(&line);
                }
                0
            }
            Err(_) => 1,
        },
        "nvm_print_formatted_alias" => {
            let style = alias::AliasStyle {
                default: nvm.vars.get("DEFAULT") == "true",
                lts: nvm.vars.get("NVM_LTS") == "true",
            };
            let current = nvm.vars.get("NVM_CURRENT").to_string();
            alias::print_formatted_alias(nvm, arg(0), arg(1), Some(arg(2)), style, &current);
            0
        }
        "nvm_print_alias_path" => {
            if arg(0).is_empty() {
                nvm.io.err(crate::text::messages::ALIAS_DIR_REQUIRED);
                return 1;
            }
            if arg(1).is_empty() {
                nvm.io.err(crate::text::messages::ALIAS_PATH_REQUIRED);
                return 2;
            }
            let style = alias::AliasStyle {
                default: false,
                lts: nvm.vars.get("NVM_LTS") == "true",
            };
            let current = nvm.vars.get("NVM_CURRENT").to_string();
            let no_colors = nvm.vars.get("NVM_NO_COLORS").to_string();
            let lts = nvm.vars.get("NVM_LTS").to_string();
            if alias::print_alias_path(nvm, arg(0), arg(1), style, &current) {
                nvm.prefix_assignments(&[
                    ("NVM_NO_COLORS", &no_colors),
                    ("NVM_LTS", &lts),
                    ("DEFAULT", "false"),
                ]);
            }
            0
        }
        "nvm_print_default_alias" => {
            if arg(0).is_empty() {
                nvm.io.err(crate::text::messages::DEFAULT_ALIAS_REQUIRED);
                return 1;
            }
            let current = nvm.vars.get("NVM_CURRENT").to_string();
            let no_colors = nvm.vars.get("NVM_NO_COLORS").to_string();
            if alias::print_default_alias(nvm, arg(0), &current) {
                nvm.prefix_assignments(&[("NVM_NO_COLORS", &no_colors), ("DEFAULT", "true")]);
            }
            0
        }
        "nvm_get_checksum_binary" => match checksum::get_checksum_binary(nvm) {
            Ok(program) => {
                nvm.io.echo(program);
                0
            }
            Err(status) => status,
        },
        "nvm_get_checksum_alg" => match checksum::get_checksum_alg(nvm) {
            Ok(algorithm) => {
                nvm.io.echo(algorithm);
                0
            }
            Err(status) => status,
        },
        "nvm_compute_checksum" => {
            let (sum, status) = checksum::compute_checksum(nvm, arg(0));
            nvm.io.out_raw(&sum);
            status
        }
        "nvm_compare_checksum" => checksum::compare_checksum(nvm, arg(0), arg(1)),
        "nvm_get_checksum" => {
            let (sum, status) = checksum::get_checksum(nvm, arg(0), arg(1), arg(2), arg(3), arg(4));
            nvm.io.out_raw(&sum);
            status
        }
        "nvm_get_download_slug" => match artifact::get_download_slug(nvm, arg(0), arg(1), arg(2)) {
            Ok(slug) => {
                nvm.io.echo(&slug);
                0
            }
            Err(status) => status,
        },
        "nvm_get_artifact_compression" => {
            let compression = artifact::get_artifact_compression(nvm, arg(0));
            nvm.io.echo(compression);
            0
        }
        "nvm_download_artifact" => artifact::download_artifact(nvm, arg(0), arg(1), arg(2), arg(3)),
        "nvm_extract_tarball" => {
            let args: Vec<&str> = args.iter().map(String::as_str).collect();
            artifact::extract_tarball(nvm, &args)
        }
        "nvm_install_binary_extract" => {
            let args: Vec<&str> = args.iter().map(String::as_str).collect();
            artifact::install_binary_extract(nvm, &args)
        }
        "nvm_install_binary" => artifact::install_binary(nvm, arg(0), arg(1), arg(2), arg(3)),
        "nvm_get_make_jobs" => artifact::get_make_jobs(nvm, arg(0)),
        "nvm_install_source" => {
            artifact::install_source(nvm, arg(0), arg(1), arg(2), arg(3), arg(4))
        }
        "nvm_clang_version" => {
            let version = artifact::clang_version(nvm);
            if !version.is_empty() {
                nvm.io.echo(&version);
            }
            0
        }
        "nvm_validate_install" => artifact::validate_install(nvm, arg(0)),
        "nvm_ls_cached" => {
            let listing = artifact::ls_cached(nvm, arg(0));
            print(nvm, (listing, 0))
        }
        "nvm_offline_version" => {
            let (text, status) = artifact::offline_version(nvm, arg(0));
            nvm.io.echo(&text);
            status
        }
        "nvm_install_lock_name" => {
            nvm.io.out_raw(&lock::lock_name(arg(0)));
            0
        }
        "nvm_acquire_install_lock" => lock::acquire(nvm, arg(0)),
        "nvm_release_install_lock" => {
            lock::release(nvm);
            0
        }
        "nvm_use_if_needed" => {
            if arg(0) == local::ls_current(nvm) {
                return 0;
            }
            let mut words = vec!["use".to_string()];
            words.extend(args.iter().cloned());
            cmd::nvm(nvm, &words)
        }
        "nvm_ensure_default_set" => cmd::install::ensure_default_set(nvm, arg(0)),
        "nvm_install_npm_if_needed" => npm::install_npm_if_needed(nvm),
        "nvm_install_latest_npm" => npm::install_latest_npm(nvm),
        "nvm_npm_global_modules" => {
            let modules = npm::npm_global_modules(nvm, arg(0));
            nvm.io.echo(&modules);
            0
        }
        "nvm_get_default_packages" => {
            let (packages, status) = npm::get_default_packages(nvm);
            nvm.io.out_raw(&packages);
            status
        }
        "nvm_install_default_packages" => {
            let done = npm::install_default_packages(nvm);
            // Called on its own, the undeclared `EXIT_CODE` lands in the shell.
            nvm.vars.assign("EXIT_CODE", done.exit_code.to_string());
            done.status
        }
        "nvm_check_file_permissions" => {
            let (status, last) = cmd::uninstall::check_file_permissions(nvm, arg(0));
            nvm.vars.assign("FILE", last);
            status
        }
        "nvm_set_colors" => cmd::set_colors_to(nvm, arg(0)),
        "nvm_get_latest" => {
            let (latest, status) = remote::get_latest(nvm);
            if status == 0 {
                nvm.io.echo(&latest);
            }
            status
        }
        // nvm: nvm_process_nvmrc
        "nvm_process_nvmrc" => {
            let content = match std::fs::read(arg(0)) {
                Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                Err(error) => {
                    let reason = crate::text::messages::strerror(&error);
                    nvm.io.err(&format!("cat: {}: {reason}", arg(0)));
                    String::new()
                }
            };
            match nvmrc::process_content(nvm, content.trim_end_matches('\n')) {
                Some(version) => {
                    nvm.io.echo(&version);
                    0
                }
                None => 1,
            }
        }
        "__nvm_aliases" => {
            let names = alias_files(nvm);
            if !names.is_empty() {
                nvm.io.echo(&names.join("\n"));
            }
            0
        }
        "__nvm_complete" => {
            complete(nvm, args);
            0
        }
        // Every function nvm.sh defines is listed above. Anything else is
        // not nvm's, and `nvm` answers it the way it answers any command it
        // does not know.
        _ => cmd::nvm(nvm, &["__nvmimic".to_string()]),
    }
}

/// Answers one TAB: `mode` (`bash` or `zsh`), the index of the word being
/// typed, then every word on the line. Prints one suggestion per line. In
/// `zsh` mode a suggestion that has a description is `word:description`,
/// the form zsh's `_describe` reads.
///
/// nvm: bash_completion __nvm
fn complete(nvm: &mut Nvm, args: &[String]) {
    use crate::complete::{NVM, Takes, suggest};
    let describe = args.first().is_some_and(|mode| mode == "zsh");
    let current: usize = args.get(1).and_then(|n| n.parse().ok()).unwrap_or(0);
    let words = args.get(2..).unwrap_or_default();
    let root = match words.first().map(String::as_str) {
        Some(word) if word == nvm.identity.name => nvm.identity.completion,
        _ => &NVM,
    };
    let backend = nvm.backend;
    // The same words, in the same order, as nvm's own completion builds
    // from `nvm_ls` and its `__nvm_aliases`.
    let always: Vec<String> = ["node", "stable", "unstable", "iojs"]
        .into_iter()
        .filter(|name| *name != "iojs" || backend.offers(crate::complete::Needs::Iojs))
        .map(str::to_string)
        .collect();
    let mut list = |takes: Takes| -> Vec<String> {
        let mut found = Vec::new();
        if takes == Takes::Version {
            let (installed, _) = local::ls(nvm, "");
            found.extend(installed.split_whitespace().map(str::to_string));
        }
        if matches!(takes, Takes::Version | Takes::Alias) {
            found.extend(alias_files(nvm));
        }
        found.extend(always.iter().cloned());
        found
    };
    let found = suggest(
        root,
        words,
        current,
        &|needs| backend.offers(needs),
        &mut list,
    );
    let lines: Vec<String> = found
        .into_iter()
        .map(|suggestion| match (describe, suggestion.about) {
            (false, _) => suggestion.word,
            (true, "") => suggestion.word.replace(':', "\\:"),
            (true, about) => format!("{}:{about}", suggestion.word.replace(':', "\\:")),
        })
        .collect();
    if !lines.is_empty() {
        nvm.io.echo(&lines.join("\n"));
    }
}

/// The alias files under `$NVM_DIR/alias`, relative to it, in the order
/// `find "${PWD}" -type f` walks them.
///
/// nvm: bash_completion __nvm_aliases
fn alias_files(nvm: &Nvm) -> Vec<String> {
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
    names
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
        "install" => {
            let version = nvm.quiet_err(|nvm| alias::alias(nvm, "default").0);
            if !version.is_empty() && version != "N/A" && version::is_valid(&version) {
                nvm.quiet_out(|nvm| cmd::nvm(nvm, &["install".to_string(), version]))
            } else if nvm.quiet(|nvm| nvmrc::rc_version(nvm, false).1) == 0 {
                nvm.quiet_out(|nvm| cmd::nvm(nvm, &["install".to_string()]))
            } else {
                0
            }
        }
        _ => {
            nvm.io.err("Invalid auto mode supplied.");
            1
        }
    }
}
