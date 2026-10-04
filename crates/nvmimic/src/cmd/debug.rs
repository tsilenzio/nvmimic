//! `nvm debug`: prints what nvm knows about the shell and the programs it
//! depends on, for pasting into a bug report. Everything goes to stderr and
//! the exit status is always 42.

use crate::context::Nvm;
use crate::nvm::{checksum, local, paths, platform, tools};
use std::collections::BTreeMap;

/// The programs `nvm debug` reports on. `cargo xtask gen-shell` writes this
/// list into `shell/nvm.sh`, which looks each one up.
pub const TOOLS: &[&str] = &[
    "awk", "curl", "wget", "git", "grep", "sed", "cut", "basename", "rm", "mkdir", "xargs",
];

/// The line that ends each program's entry in what the shell side sends.
const END: &str = "__nvmimic_end";

/// What nvm's `nvm_command_info` says about each program the calling shell
/// can find. A shell knows things about a name that a program started from
/// it cannot see, such as an alias, so `nvm.sh` keeps nvm's own
/// `nvm_command_info`, asks it, and passes the answers along, one entry per
/// program: its name, the answer, and an end line. Programs the shell
/// cannot find are left out.
fn command_infos(listing: &str) -> BTreeMap<String, String> {
    let mut infos = BTreeMap::new();
    let mut lines = listing.lines();
    while let Some(name) = lines.next() {
        let info: Vec<&str> = lines.by_ref().take_while(|line| *line != END).collect();
        infos.insert(name.to_string(), info.join("\n"));
    }
    infos
}

/// The first line of what a program printed, like `| command head -n 1`.
fn first_line(output: &[u8]) -> String {
    String::from_utf8_lossy(output)
        .lines()
        .next()
        .unwrap_or("")
        .to_string()
}

/// nvm: nvm:debug
pub fn run(nvm: &mut Nvm, command_info: &str) -> i32 {
    let infos = command_infos(command_info);
    let var = |nvm: &Nvm, name: &str| nvm.vars.get(name).to_string();
    let shown = |nvm: &Nvm, path: &str| {
        paths::sanitize(path, nvm.vars.get("NVM_DIR"), nvm.vars.get("HOME"))
    };
    let printed = |nvm: &mut Nvm, program: &str, args: &[&str]| {
        tools::run(nvm, program, args)
            .map(|ran| ran.stdout)
            .unwrap_or_default()
    };

    nvm.io
        .err(&format!("nvm --version: v{}", crate::pin::VERSION));
    if !var(nvm, "TERM_PROGRAM").is_empty() {
        nvm.io
            .err(&format!("$TERM_PROGRAM: {}", var(nvm, "TERM_PROGRAM")));
    }
    nvm.io.err(&format!("$SHELL: {}", var(nvm, "SHELL")));
    nvm.io.err(&format!("$SHLVL: {}", var(nvm, "SHLVL")));
    let user = printed(nvm, "whoami", &[]);
    nvm.io.err(&format!(
        "whoami: '{}'",
        String::from_utf8_lossy(&user).trim_end_matches('\n')
    ));
    nvm.io.err(&format!("${{HOME}}: {}", var(nvm, "HOME")));
    let nvm_dir = shown(nvm, &var(nvm, "NVM_DIR"));
    nvm.io.err(&format!("${{NVM_DIR}}: '{nvm_dir}'"));
    let path = shown(nvm, &var(nvm, "PATH"));
    nvm.io.err(&format!("${{PATH}}: {path}"));
    let prefix = shown(nvm, &var(nvm, "PREFIX"));
    nvm.io.err(&format!("$PREFIX: '{prefix}'"));
    let npm_prefix = shown(nvm, &var(nvm, "NPM_CONFIG_PREFIX"));
    nvm.io
        .err(&format!("${{NPM_CONFIG_PREFIX}}: '{npm_prefix}'"));
    nvm.io.err(&format!(
        "$NVM_NODEJS_ORG_MIRROR: '{}'",
        var(nvm, "NVM_NODEJS_ORG_MIRROR")
    ));
    nvm.io.err(&format!(
        "$NVM_IOJS_ORG_MIRROR: '{}'",
        var(nvm, "NVM_IOJS_ORG_MIRROR")
    ));

    // `${SHELL} --version`, with the value split into words.
    let shell = var(nvm, "SHELL");
    let mut words = shell.split_whitespace();
    let shell_version = match words.next() {
        Some(program) => {
            let mut args: Vec<&str> = words.collect();
            args.push("--version");
            first_line(&printed(nvm, program, &args))
        }
        None => String::new(),
    };
    nvm.io.err(&format!("shell version: '{shell_version}'"));

    // `uname -a` without its second word, the machine's name.
    let uname = printed(nvm, "uname", &["-a"]);
    let uname = String::from_utf8_lossy(&uname)
        .split_whitespace()
        .enumerate()
        .filter(|(index, _)| *index != 1)
        .map(|(_, word)| word.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    nvm.io.err(&format!("uname -a: '{uname}'"));

    let checksum_binary = nvm
        .quiet_err(checksum::get_checksum_binary)
        .unwrap_or_default();
    nvm.io.err(&format!("checksum binary: '{checksum_binary}'"));

    // `local OS_VERSION` has no value, so dash starts from the caller's.
    let saved = nvm.local("OS_VERSION");
    let mut os_version = var(nvm, "OS_VERSION");
    nvm.restore("OS_VERSION", saved);
    let os = platform::get_os(nvm);
    if os == "darwin" && paths::on_path(nvm.vars.get("PATH"), "sw_vers") {
        let listing = printed(nvm, "sw_vers", &[]);
        os_version = String::from_utf8_lossy(&listing)
            .lines()
            .filter_map(|line| line.split_whitespace().nth(1))
            .collect::<Vec<_>>()
            .join(" ");
    } else if let Ok(issue) = std::fs::read_to_string("/etc/issue") {
        // `sed 's/\\.//g'`: drops each backslash with the character after it.
        let mut first = String::new();
        let mut chars = issue.lines().next().unwrap_or("").chars();
        while let Some(c) = chars.next() {
            match c {
                '\\' if chars.clone().next().is_some() => {
                    chars.next();
                }
                c => first.push(c),
            }
        }
        os_version = first;
        if os_version.is_empty()
            && let Ok(release) = std::fs::read_to_string("/etc/os-release")
        {
            let field = |name: &str| {
                release
                    .lines()
                    .find_map(|line| line.strip_prefix(&format!("{name}=")))
                    .map(|value| value.trim_matches(['"', '\'']).to_string())
                    .unwrap_or_default()
            };
            os_version = format!("{} {}", field("NAME"), field("VERSION"));
        }
    }
    if !os_version.is_empty() {
        nvm.io.err(&format!("OS version: {os_version}"));
    }

    let info = |name: &str| infos.get(name).cloned();
    match info("awk") {
        Some(info) => {
            // GNU awk answers `--version`, others `-W version`.
            let version = match tools::run_quiet(nvm, "awk", &["--version"]) {
                Some(ran) if ran.ok() => ran.stdout,
                _ => printed(nvm, "awk", &["-W", "version"]),
            };
            nvm.io
                .err(&format!("awk: {info}, {}", first_line(&version)));
        }
        None => nvm.io.err("awk: not found"),
    }
    for (name, flag) in [("curl", "-V"), ("wget", "-V")] {
        match info(name) {
            Some(info) => {
                let version = printed(nvm, name, &[flag]);
                nvm.io
                    .err(&format!("{name}: {info}, {}", first_line(&version)));
            }
            None => nvm.io.err(&format!("{name}: not found")),
        }
    }

    // On macOS and FreeBSD these six have no `--version`, so nvm only says
    // where they are. Elsewhere they get the full treatment below.
    let additional = ["sed", "cut", "basename", "rm", "mkdir", "xargs"];
    let mut versioned = vec!["git", "grep"];
    if os != "darwin" && os != "freebsd" {
        versioned.extend(additional);
    } else {
        for name in additional {
            match info(name) {
                Some(info) => nvm.io.err(&format!("{name}: {info}")),
                None => nvm.io.err(&format!("{name}: not found")),
            }
        }
    }
    for name in &versioned {
        match info(name) {
            Some(info) => {
                // busybox applets have no `--version` either. nvm spots one
                // by `ls -l` of the first word of the info, and when that
                // word is not a path, what `ls` says about it shows.
                let place = info.split_whitespace().next().unwrap_or("");
                let listed = printed(nvm, "ls", &["-l", place]);
                let busybox = String::from_utf8_lossy(&listed).contains("busybox");
                let flag = if busybox { "--help" } else { "--version" };
                let version = tools::run_merged(nvm, name, &[flag])
                    .map(|ran| ran.stdout)
                    .unwrap_or_default();
                nvm.io
                    .err(&format!("{name}: {info}, {}", first_line(&version)));
            }
            None => nvm.io.err(&format!("{name}: not found")),
        }
    }
    // Both loop variables are assigned without `local` and stay set.
    if let Some(last) = versioned.last() {
        nvm.vars.assign("tool", *last);
    }

    let commands: [(&str, &[&str]); 6] = [
        ("nvm", &["current"]),
        ("which", &["node"]),
        ("which", &["iojs"]),
        ("which", &["npm"]),
        ("npm", &["config", "get", "prefix"]),
        ("npm", &["root", "-g"]),
    ];
    for (program, args) in commands {
        let output = match program {
            "nvm" => local::ls_current(nvm),
            // zsh has its own `which`, which says so when it finds nothing.
            "which" if nvm.dialect.is_zsh() => match paths::which(nvm.vars.get("PATH"), args[0]) {
                Some(found) => found,
                None => format!("{} not found", args[0]),
            },
            _ => tools::run_merged(nvm, program, args)
                .map(|ran| ran.text())
                .unwrap_or_default(),
        };
        let line = format!("{program} {}", args.join(" "));
        let output = shown(nvm, &output);
        nvm.io.err(&format!("{line}: {output}"));
        nvm.vars.assign("NVM_DEBUG_COMMAND", line);
    }
    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_end_at_the_end_line() {
        let listing = "awk\n/usr/bin/awk\n__nvmimic_end\nls\n/bin/ls (ls -G)\n__nvmimic_end\n";
        let infos = command_infos(listing);
        assert_eq!(infos["awk"], "/usr/bin/awk");
        assert_eq!(infos["ls"], "/bin/ls (ls -G)");
        assert_eq!(infos.len(), 2);
    }
}
