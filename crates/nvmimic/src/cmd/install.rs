//! `nvm install`: works out which version is meant, downloads it or builds
//! it, switches to it, and sets up aliases and packages around it.

use crate::context::Nvm;
use crate::nvm::{
    alias, artifact, local, lock, npm, nvmrc, paths, platform, remote, tools, version,
};
use crate::text::install as text;
use crate::text::messages;

/// Variables `nvm install` declares `local` that other nvm functions read
/// while it runs, or that dash fills from the caller's variables of the
/// same name because nvm declares them without a value.
const LOCALS: &[&str] = &[
    "NVM_OFFLINE",
    "LTS",
    "ALIAS",
    "PROVIDED_REINSTALL_PACKAGES_FROM",
    "REINSTALL_PACKAGES_FROM",
    "SKIP_DEFAULT_PACKAGES",
    "provided_version",
];

/// The values nvm puts in front of its calls to `nvm_install_binary` and
/// `nvm_install_source`, as `NVM_NO_PROGRESS=... NVM_OFFLINE=... function`.
struct Prefixed {
    no_progress: String,
    offline: String,
}

/// nvm: nvm:install
pub fn run(nvm: &mut Nvm, args: &[String]) -> i32 {
    let mut prefixed = None;
    let status = nvm.scoped(LOCALS, |nvm| install(nvm, args, &mut prefixed));
    // Under bash's POSIX mode those two assignments outlive the call, and
    // `nvm install`'s own `local NVM_OFFLINE` does not stop them.
    if let Some(prefixed) = prefixed {
        nvm.prefix_assignments(&[
            ("NVM_NO_PROGRESS", &prefixed.no_progress),
            ("NVM_OFFLINE", &prefixed.offline),
        ]);
    }
    status
}

fn call(nvm: &mut Nvm, words: &[&str]) -> i32 {
    let args: Vec<String> = words.iter().map(|word| word.to_string()).collect();
    super::nvm(nvm, &args)
}

/// `--reinstall-packages-from=<version>` and `--copy-packages-from=<version>`.
/// Returns the status to stop with when the option is not acceptable.
fn packages_from(
    nvm: &mut Nvm,
    arg: &str,
    provided: &mut String,
    resolved: &mut String,
) -> Result<(), i32> {
    let (value, repeated, missing) = match arg.strip_prefix("--reinstall-packages-from=") {
        Some(value) => (value, text::REINSTALL_ONCE, text::REINSTALL_NEEDS_VERSION),
        None => (
            arg.strip_prefix("--copy-packages-from=").unwrap_or(""),
            text::REINSTALL_ONCE_OR_COPY,
            text::COPY_NEEDS_VERSION,
        ),
    };
    if !provided.is_empty() {
        nvm.io.err(repeated);
        return Err(6);
    }
    if value.is_empty() {
        nvm.io.err(missing);
        return Err(6);
    }
    *provided = value.to_string();
    *resolved = local::version(nvm, value).0;
    Ok(())
}

fn install(nvm: &mut Nvm, args: &[String], prefixed: &mut Option<Prefixed>) -> i32 {
    let version_not_provided = args.is_empty();
    let os = platform::get_os(nvm);

    let mut nobinary = false;
    let mut nosource = false;
    let mut noprogress = false;
    nvm.vars.set_quiet("NVM_OFFLINE", "0");
    // Declared without a value, so under dash these start out as whatever
    // the caller's variables of the same names hold.
    let mut lts = nvm.vars.get("LTS").to_string();
    let mut alias_name = nvm.vars.get("ALIAS").to_string();
    let mut provided_packages_from = nvm.vars.get("PROVIDED_REINSTALL_PACKAGES_FROM").to_string();
    let mut packages_from_version = nvm.vars.get("REINSTALL_PACKAGES_FROM").to_string();
    let mut skip_default_packages = !nvm.vars.get("SKIP_DEFAULT_PACKAGES").is_empty();
    let mut upgrade_npm = false;
    let mut write_nvmrc = false;

    let mut rest = args;
    while let Some(arg) = rest.first() {
        let mut consumed = 1;
        match arg.as_str() {
            typo if typo.starts_with("---") => {
                nvm.io.err(text::TRIPLE_DASH);
                return 55;
            }
            "-s" => {
                nobinary = true;
                if nosource {
                    nvm.io.err(text::SOURCE_AND_BINARY);
                    return 6;
                }
            }
            "-b" => {
                nosource = true;
                if nobinary {
                    nvm.io.err(text::SOURCE_AND_BINARY);
                    return 6;
                }
            }
            "-j" => {
                let jobs = rest.get(1).map_or("", String::as_str);
                artifact::get_make_jobs(nvm, jobs);
                consumed = rest.len().min(2);
            }
            "--no-progress" => noprogress = true,
            "--offline" => nvm.vars.set_quiet("NVM_OFFLINE", "1"),
            "--lts" => lts = "*".to_string(),
            "--latest-npm" => upgrade_npm = true,
            "--default" => {
                if !alias_name.is_empty() {
                    nvm.io.err(text::DEFAULT_AND_ALIAS);
                    return 6;
                }
                alias_name = "default".to_string();
            }
            "--reinstall-packages-from" | "--copy-packages-from" => {
                nvm.io.err(&text::needs_equals(arg));
                return 6;
            }
            "--skip-default-packages" => skip_default_packages = true,
            "--save" | "-w" => {
                if write_nvmrc {
                    nvm.io.err(messages::SAVE_ONCE);
                    return 6;
                }
                write_nvmrc = true;
            }
            other => {
                if let Some(name) = other.strip_prefix("--lts=") {
                    lts = name.to_string();
                } else if let Some(name) = other.strip_prefix("--alias=") {
                    if !alias_name.is_empty() {
                        nvm.io.err(text::DEFAULT_AND_ALIAS);
                        return 6;
                    }
                    alias_name = name.to_string();
                } else if other.starts_with("--reinstall-packages-from=")
                    || other.starts_with("--copy-packages-from=")
                {
                    if let Err(status) = packages_from(
                        nvm,
                        other,
                        &mut provided_packages_from,
                        &mut packages_from_version,
                    ) {
                        return status;
                    }
                } else {
                    break;
                }
            }
        }
        rest = &rest[consumed..];
    }
    nvm.vars.set_quiet("ALIAS", &alias_name);

    // NVM_NO_SOURCE_FALLBACK=1 is `-b` for every install.
    if nvm.vars.get("NVM_NO_SOURCE_FALLBACK") == "1" && !nosource {
        if nobinary {
            nvm.io.err(text::SOURCE_WITH_NO_FALLBACK);
            return 6;
        }
        nosource = true;
    }

    let offline = nvm.vars.get("NVM_OFFLINE") == "1";
    let path = nvm.vars.get("PATH").to_string();
    if !offline && !paths::on_path(&path, "curl") && !paths::on_path(&path, "wget") {
        nvm.io.err(text::NEEDS_CURL_OR_WGET);
        return 1;
    }

    let mut provided = rest.first().cloned().unwrap_or_default();
    let silent = nvm.vars.get("NVM_SILENT") == "1";
    if !provided.is_empty() {
        rest = &rest[1..];
    } else if !lts.is_empty() {
        match lts.as_str() {
            "*" => nvm.io.echo(text::INSTALLING_LATEST_LTS),
            name => nvm.io.echo(&text::installing_lts_line(name)),
        }
        rest = rest.get(1..).unwrap_or(&[]);
    } else {
        provided = nvmrc::rc_version(nvm, silent).0.unwrap_or_default();
        if version_not_provided && provided.is_empty() {
            for line in text::INSTALL_USAGE {
                nvm.io.err(line);
            }
            return 127;
        }
    }

    if provided == "lts/*" {
        lts = "*".to_string();
        provided.clear();
    } else if let Some(name) = provided.strip_prefix("lts/") {
        lts = name.to_string();
        provided.clear();
    }
    nvm.vars.set_quiet("provided_version", &provided);

    let (version, found) = if offline {
        let mut pattern = provided.clone();
        if !lts.is_empty() {
            let name = format!("lts/{lts}");
            pattern = nvm.quiet_err(|nvm| alias::resolve_alias(nvm, &name)).0;
            if pattern.is_empty() {
                nvm.io.err(&text::lts_alias_not_local(&lts));
                return 3;
            }
        }
        artifact::offline_version(nvm, &pattern)
    } else {
        nvm.subshell(|nvm| {
            nvm.vars.set_quiet("NVM_VERSION_ONLY", "true");
            nvm.vars.set_quiet("NVM_LTS", &lts);
            remote::remote_version(nvm, &provided)
        })
    };

    if version == "N/A" || found != 0 {
        let mut filter = String::new();
        let command = if lts == "*" {
            filter = text::LTS_FILTER_ANY.to_string();
            "nvm ls-remote --lts".to_string()
        } else if !lts.is_empty() {
            filter = text::lts_filter(&lts);
            let command = format!("nvm ls-remote --lts={lts}");
            if provided.is_empty() {
                nvm.io.err(&text::lts_version_not_found(&lts, &command));
                return 3;
            }
            command
        } else if offline {
            "nvm ls".to_string()
        } else {
            "nvm ls-remote".to_string()
        };
        nvm.io.err(&match offline {
            true => text::version_not_found_offline(&provided, &filter, &command),
            false => text::version_not_found(&provided, &filter, &command),
        });
        return 3;
    }

    // Whatever follows the version goes to `./configure` in a source build.
    let mut additional_parameters = String::new();
    for arg in rest {
        if arg.starts_with("--reinstall-packages-from=") || arg.starts_with("--copy-packages-from=")
        {
            if let Err(status) = packages_from(
                nvm,
                arg,
                &mut provided_packages_from,
                &mut packages_from_version,
            ) {
                return status;
            }
        } else if arg == "--reinstall-packages-from" || arg == "--copy-packages-from" {
            nvm.io.err(&text::needs_equals(arg));
            return 6;
        } else if arg == "--skip-default-packages" {
            skip_default_packages = true;
        } else {
            additional_parameters.push(' ');
            additional_parameters.push_str(arg);
        }
    }

    if !provided_packages_from.is_empty()
        && version::ensure_prefix(&provided_packages_from) == version
    {
        nvm.io.err(text::SAME_VERSION_REINSTALL);
        return 4;
    } else if packages_from_version == "N/A" {
        nvm.io.err(text::REINSTALL_NEEDS_VERSION);
        return 5;
    }
    let reinstall_packages = (!packages_from_version.is_empty() && packages_from_version != "N/A")
        .then_some(packages_from_version.as_str());

    let flavor = match version::is_iojs(&version) {
        true => version::IOJS_PREFIX,
        false => version::NODE_PREFIX,
    };

    let mut exit_code;
    if local::is_version_installed(nvm, &version) && artifact::validate_install(nvm, &version) == 0
    {
        nvm.io.err(&text::already_installed(&version));
        exit_code = call(nvm, &["use", &version]);
        if exit_code == 0 {
            exit_code = after_use(
                nvm,
                upgrade_npm,
                skip_default_packages,
                reinstall_packages,
                exit_code,
            );
        }
        if !lts.is_empty() {
            ensure_default_set(nvm, &format!("lts/{}", lts.to_ascii_lowercase()));
        } else {
            ensure_default_set(nvm, &provided);
        }
        if write_nvmrc {
            exit_code = nvmrc::write_nvmrc(nvm, &version, silent);
        }
        if exit_code == 0 && !alias_name.is_empty() {
            exit_code = call(nvm, &["alias", &alias_name, &provided]);
        }
        return exit_code;
    }

    let hook = nvm.vars.get("NVM_INSTALL_THIRD_PARTY_HOOK").to_string();
    if !hook.is_empty() {
        nvm.io.err(text::HOOK_DISPATCH);
        let method = if nobinary { "source" } else { "binary" };
        let version_path = paths::version_path(nvm, &version).unwrap_or_else(|message| {
            nvm.io.err(&message);
            String::new()
        });
        let status = match tools::command(nvm, &hook) {
            Some(mut command) => {
                command.args([version.as_str(), flavor, "std", method, &version_path]);
                tools::run_through(nvm, command)
            }
            None => 127,
        };
        if status != 0 {
            nvm.io.err(text::HOOK_FAILED);
            return status;
        }
        if !local::is_version_installed(nvm, &version) {
            nvm.io.err(text::HOOK_DID_NOT_INSTALL);
            return 33;
        }
        exit_code = 0;
    } else {
        // One install of a version at a time, so two runs cannot both work
        // on its directory.
        if lock::acquire(nvm, &version) != 0 {
            return 1;
        }
        match os.as_str() {
            "freebsd" => {
                nobinary = true;
                nvm.io.err(text::NO_FREEBSD_BINARY);
            }
            "openbsd" => {
                nobinary = true;
                nvm.io.err(text::NO_OPENBSD_BINARY);
            }
            "sunos" if platform::has_solaris_binary(&version) != 0 => {
                nobinary = true;
                nvm.io.err(&text::no_sunos_binary(&version));
            }
            _ => {}
        }

        let values = Prefixed {
            no_progress: match nvm.vars.get("NVM_NO_PROGRESS") {
                "" => (noprogress as u8).to_string(),
                set => set.to_string(),
            },
            offline: nvm.vars.get("NVM_OFFLINE").to_string(),
        };
        let with_prefix = |nvm: &mut Nvm, body: &mut dyn FnMut(&mut Nvm) -> i32| -> i32 {
            let before = nvm.vars.get_opt("NVM_NO_PROGRESS").map(str::to_string);
            nvm.vars.set_quiet("NVM_NO_PROGRESS", &values.no_progress);
            let status = body(nvm);
            match before {
                Some(value) => nvm.vars.set_quiet("NVM_NO_PROGRESS", &value),
                None => nvm.vars.unset_quiet("NVM_NO_PROGRESS"),
            }
            status
        };

        let mut ran_installer = false;
        if !nobinary && platform::binary_available(&version) == 0 {
            let no_source = if nosource { "1" } else { "0" };
            exit_code = with_prefix(nvm, &mut |nvm| {
                artifact::install_binary(nvm, flavor, "std", &version, no_source)
            });
            ran_installer = true;
        } else {
            exit_code = -1;
            if nosource {
                nvm.io.err(&text::binary_not_available(&version));
                exit_code = 3;
            }
        }

        if exit_code != 0 && !nosource {
            if nvm.vars.get("NVM_MAKE_JOBS").is_empty() {
                artifact::get_make_jobs(nvm, "");
            }
            if os == "win" {
                nvm.io.err(text::NO_SOURCE_ON_WINDOWS);
                exit_code = 87;
            } else {
                let jobs = nvm.vars.get("NVM_MAKE_JOBS").to_string();
                exit_code = with_prefix(nvm, &mut |nvm| {
                    artifact::install_source(
                        nvm,
                        flavor,
                        "std",
                        &version,
                        &jobs,
                        &additional_parameters,
                    )
                });
                ran_installer = true;
            }
        }
        if ran_installer {
            *prefixed = Some(values);
        }
        lock::release(nvm);
    }

    if exit_code == 0 && artifact::validate_install(nvm, &version) != 0 {
        nvm.io.err(&text::failed_verification(&version));
        exit_code = 1;
    }

    if exit_code == 0 {
        let mut ready = use_if_needed(nvm, &version);
        if ready == 0 {
            ready = npm::install_npm_if_needed(nvm);
        }
        if ready == 0 {
            if !lts.is_empty() {
                ensure_default_set(nvm, &format!("lts/{lts}"));
            } else {
                ensure_default_set(nvm, &provided);
            }
            exit_code = after_use(
                nvm,
                upgrade_npm,
                skip_default_packages,
                reinstall_packages,
                exit_code,
            );
        } else {
            exit_code = ready;
        }
    }
    exit_code
}

/// The optional steps once the version is active: the newest npm, the
/// default packages, and the packages of another version. Returns the new
/// value of `nvm install`'s `EXIT_CODE`.
fn after_use(
    nvm: &mut Nvm,
    upgrade_npm: bool,
    skip_default_packages: bool,
    reinstall_packages: Option<&str>,
    mut exit_code: i32,
) -> i32 {
    if upgrade_npm {
        exit_code = call(nvm, &["install-latest-npm"]);
    }
    if exit_code == 0 && !skip_default_packages {
        exit_code = npm::install_default_packages(nvm).exit_code;
    }
    if exit_code == 0
        && let Some(from) = reinstall_packages
    {
        exit_code = call(nvm, &["reinstall-packages", from]);
    }
    exit_code
}

/// nvm: nvm_use_if_needed
pub fn use_if_needed(nvm: &mut Nvm, version: &str) -> i32 {
    if version == local::ls_current(nvm) {
        return 0;
    }
    call(nvm, &["use", version])
}

/// Makes `version` the default alias when there is no default yet.
///
/// nvm: nvm_ensure_default_set
pub fn ensure_default_set(nvm: &mut Nvm, version: &str) -> i32 {
    if version.is_empty() {
        nvm.io.err(text::DEFAULT_SET_VERSION_REQUIRED);
        return 1;
    } else if nvm.quiet(|nvm| alias::alias(nvm, "default").1) == 0 {
        return 0;
    }
    let mut status = 0;
    let output = nvm.capture(|nvm| status = call(nvm, &["alias", "default", version]));
    nvm.io.echo(&text::creating_default_alias(&output));
    status
}
