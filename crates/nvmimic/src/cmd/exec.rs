//! `nvm exec` and `nvm run`: a command under a chosen version, through
//! `$NVM_DIR/nvm-exec` as nvm runs it.

use crate::context::Nvm;
use crate::nvm::{local, npm, nvmrc, version};
use crate::text::messages;

/// `$(nvm use --silent "${VERSION}" && nvm_print_npm_version)`: switches in
/// a subshell, so what `use` prints to stderr still shows, and gives the npm
/// suffix only when the switch worked.
fn npm_after_use(nvm: &mut Nvm, version: &str) -> String {
    nvm.subshell(|nvm| {
        let args = [
            "use".to_string(),
            "--silent".to_string(),
            version.to_string(),
        ];
        let status = nvm.quiet_out(|nvm| super::nvm(nvm, &args));
        if status == 0 {
            npm::print_npm_version(nvm)
        } else {
            String::new()
        }
    })
}

/// nvm: nvm:exec
pub fn exec(nvm: &mut Nvm, args: &[String]) -> i32 {
    let mut silent = false;
    let mut lts: Option<String> = None;
    let mut rest = args;
    while let Some(arg) = rest.first() {
        match arg.as_str() {
            "--silent" => silent = true,
            "--lts" => lts = Some("*".to_string()),
            "--" => break,
            other if other.starts_with("--lts=") => lts = Some(other["--lts=".len()..].to_string()),
            other if other.starts_with("--") => {
                nvm.io.err(&messages::unsupported_option(other));
                return 55;
            }
            "" => {}
            _ => break,
        }
        rest = &rest[1..];
    }

    let silent = nvm.silent(silent);
    let mut provided = rest.first().cloned().unwrap_or_default();
    let mut version = String::new();
    let mut source = "";
    if let Some(lts) = lts.as_deref().filter(|lts| !lts.is_empty()) {
        provided = format!("lts/{lts}");
        version = provided.clone();
        source = "lts";
    } else if !provided.is_empty() {
        version = local::version(nvm, &provided).0;
        if version == "N/A" && !version::is_valid(&provided) {
            let (rc, status) = nvmrc::rc_version(nvm, silent);
            if status == 0 {
                nvm.vars.assign("has_checked_nvmrc", "1");
            }
            provided = rc.unwrap_or_default();
            version = local::version(nvm, &provided).0;
            if !provided.is_empty() {
                source = "nvmrc";
            }
        } else {
            source = "arg";
            rest = &rest[1..];
        }
    }

    if source.is_empty() {
        if !silent {
            for line in messages::EXEC_NO_VERSION {
                nvm.io.err(line);
            }
        }
        provided = "current".to_string();
        version = local::version(nvm, "current").0;
    }

    let status = local::ensure_version_installed(nvm, &provided, false);
    nvm.vars.assign("EXIT_CODE", status.to_string());
    if status != 0 {
        return status;
    }

    if !silent {
        let npm = npm_after_use(nvm, &version);
        let resolved = local::version(nvm, &version).0;
        let line = match lts.as_deref() {
            Some("*") => messages::running_latest_lts(&resolved, &npm),
            Some(lts) if !lts.is_empty() => messages::running_lts(lts, &resolved, &npm),
            _ if version::is_iojs(&version) => {
                messages::running_iojs(version::strip_iojs_prefix(&version), &npm)
            }
            _ => messages::running_node(&version, &npm),
        };
        nvm.io.echo(&line);
    }

    nvm.io.flush();
    let nvm_exec = format!("{}/nvm-exec", nvm.vars.get("NVM_DIR"));
    let status = std::process::Command::new(&nvm_exec)
        .args(rest)
        .env_clear()
        .envs(nvm.vars.exported())
        .env("NODE_VERSION", &version)
        .current_dir(nvm.vars.get("PWD"))
        .status();
    match status {
        Ok(status) => exit_status(status),
        Err(error) => {
            let reason = messages::strerror(&error);
            nvm.io.err(&format!("nvm: {nvm_exec}: {reason}"));
            126
        }
    }
}

/// A child's exit status as a shell reports it: 128 plus the signal number
/// when it was killed by one.
fn exit_status(status: std::process::ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
}

/// nvm: nvm:run
pub fn run(nvm: &mut Nvm, args: &[String]) -> i32 {
    nvm.scoped(&["EXIT_CODE", "has_checked_nvmrc"], |nvm| {
        run_inner(nvm, args)
    })
}

fn run_inner(nvm: &mut Nvm, args: &[String]) -> i32 {
    let mut silent = false;
    let mut lts: Option<String> = None;
    let mut rest = args;
    while let Some(arg) = rest.first() {
        match arg.as_str() {
            "--silent" => silent = true,
            "--lts" => lts = Some("*".to_string()),
            other if other.starts_with("--lts=") => lts = Some(other["--lts=".len()..].to_string()),
            "" => {}
            _ => break,
        }
        rest = &rest[1..];
    }

    let silent = nvm.silent(silent);
    let mut checked_nvmrc = false;
    let mut rc_version: Option<String> = None;
    let mut from_nvmrc = false;
    let mut version = String::new();
    let lts_given = lts.as_deref().is_some_and(|lts| !lts.is_empty());

    if rest.is_empty() && !lts_given {
        let (rc, status) = nvmrc::rc_version(nvm, silent);
        checked_nvmrc = status == 0;
        if let Some(rc) = rc.as_deref().filter(|rc| !rc.is_empty()) {
            version = local::version(nvm, rc).0;
        }
        rc_version = rc;
        if version.is_empty() || version == "N/A" {
            for line in messages::RUN_USAGE {
                nvm.io.err(line);
            }
            return 127;
        }
    }

    let mut provided = String::new();
    if !lts_given {
        provided = rest.first().cloned().unwrap_or_default();
        if !provided.is_empty() {
            version = local::version(nvm, &provided).0;
            if version == "N/A" && !version::is_valid(&provided) {
                provided.clear();
                if !checked_nvmrc {
                    let (rc, status) = nvmrc::rc_version(nvm, silent);
                    checked_nvmrc = status == 0;
                    rc_version = rc;
                }
                if rc_version.as_deref().is_none_or(str::is_empty) {
                    if !silent {
                        for line in messages::RUN_NO_VERSION {
                            nvm.io.err(line);
                        }
                    }
                    rc_version = Some(local::version(nvm, "current").0);
                }
                // `run` assigns NVM_RC_VERSION here without declaring it.
                let rc = rc_version.clone().unwrap_or_default();
                nvm.vars.assign("NVM_RC_VERSION", rc.clone());
                provided = rc.clone();
                from_nvmrc = true;
                version = local::version(nvm, &rc).0;
            } else {
                rest = &rest[1..];
            }
        }
    }
    let _ = checked_nvmrc;

    let iojs = version::is_iojs(&version);
    let mut lts_arg = String::new();
    if let Some(lts) = lts.as_deref().filter(|lts| !lts.is_empty()) {
        lts_arg = format!("--lts={lts}");
        version.clear();
    }

    if version == "N/A" {
        return local::ensure_version_installed(nvm, &provided, from_nvmrc);
    }
    let mut exec_args = vec![
        if silent {
            "--silent".to_string()
        } else {
            String::new()
        },
        lts_arg,
        version,
        if iojs {
            "iojs".to_string()
        } else {
            "node".to_string()
        },
    ];
    exec_args.extend(rest.iter().cloned());
    let mut full = vec!["exec".to_string()];
    full.extend(exec_args);
    super::nvm(nvm, &full)
}
