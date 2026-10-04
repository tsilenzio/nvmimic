//! The `nvm` command: its help scan, then one module per subcommand.

pub mod alias;
pub mod cache;
pub mod deactivate;
pub mod debug;
pub mod exec;
pub mod install;
pub mod ls;
pub mod packages;
pub mod remote;
pub mod uninstall;
pub mod unload;
pub mod use_;
pub mod which;

use crate::context::Nvm;
use crate::nvm::{colors, local};
use crate::text::{help, messages};

/// Runs `nvm <args>` and returns its exit status. Changes to the calling
/// shell collect in `nvm.vars`.
///
/// nvm: nvm
pub fn nvm(nvm: &mut Nvm, args: &[String]) -> i32 {
    if args.is_empty() {
        return self::nvm(nvm, &["--help".to_string()]);
    }

    for arg in args {
        match arg.as_str() {
            "--" => break,
            "-h" | "help" | "--help" => {
                print_help(nvm, args);
                return 0;
            }
            _ => {}
        }
    }

    let command = args[0].as_str();
    let rest = &args[1..];
    match command {
        "deactivate" => deactivate::run(nvm, rest),
        "use" => use_::run(nvm, rest),
        "exec" => exec::exec(nvm, rest),
        "run" => exec::run(nvm, rest),
        "ls" | "list" => ls::run(nvm, rest),
        // nvm: nvm:current
        "current" => {
            nvm.io.echo(&local::ls_current(nvm));
            0
        }
        "which" => which::run(nvm, rest),
        "alias" => alias::run(nvm, rest),
        "unalias" => alias::unalias(nvm, rest),
        // nvm: nvm:version
        "version" => {
            let (version, status) = local::version(nvm, rest.first().map_or("", String::as_str));
            nvm.io.echo(&version);
            status
        }
        // nvm: nvm:--version
        "--version" | "-v" => {
            nvm.io.echo(crate::pin::VERSION);
            0
        }
        "unload" => unload::run(nvm),
        "set-colors" => set_colors(nvm, rest),
        "ls-remote" | "list-remote" => remote::ls_remote(nvm, rest),
        "version-remote" => remote::version_remote(nvm, rest),
        "cache" => cache::run(nvm, rest),
        "clear-cache" => cache::clear_cache(nvm),
        "install" | "i" => install::run(nvm, rest),
        "uninstall" => uninstall::run(nvm, rest),
        "debug" => {
            let command_info = nvm.command_info.clone();
            debug::run(nvm, &command_info)
        }
        "install-latest-npm" => packages::install_latest_npm(nvm, rest),
        "reinstall-packages" | "copy-packages" => packages::reinstall_packages(nvm, command, rest),
        // nvm: nvm:*
        _ => {
            nvm.to_stderr(|nvm| print_help(nvm, &["--help".to_string()]));
            127
        }
    }
}

/// `nvm --help`. It leaves `NVM_NO_COLORS`, `NVM_VERSION` and the loop
/// variable `j` set in the calling shell.
///
/// nvm: nvm:help
fn print_help(nvm: &mut Nvm, args: &[String]) {
    let mut no_colors = String::new();
    let mut last = String::new();
    for arg in args {
        last = arg.clone();
        if arg == "--no-colors" {
            no_colors = arg.clone();
            break;
        }
    }
    nvm.vars.assign("NVM_NO_COLORS", no_colors);
    if !args.is_empty() {
        nvm.vars.assign("j", last);
    }
    nvm.vars.assign("NVM_VERSION", crate::pin::VERSION);

    for line in help::BEFORE_LEGEND {
        let line = line
            .replace("{VERSION}", crate::pin::VERSION)
            .replace("{NODE}", crate::nvm::version::NODE_PREFIX)
            .replace("{IOJS}", crate::nvm::version::IOJS_PREFIX);
        nvm.io.echo(&line);
    }
    print_color_legend(nvm);
    for line in help::AFTER_LEGEND {
        nvm.io.echo(line);
    }
}

/// The color section of `nvm --help`.
///
/// nvm: nvm_print_color_legend
pub fn print_color_legend(nvm: &mut Nvm) {
    let has_colors = colors::has_colors(nvm);
    let saved = nvm.local("NVM_HAS_COLORS");
    nvm.vars
        .set_quiet("NVM_HAS_COLORS", if has_colors { "1" } else { "0" });
    let wrap = |nvm: &mut Nvm, letter: &str, text: &str| -> String {
        // Each piece is built in `$( ... )`, which cannot see the terminal.
        let tty = std::mem::replace(&mut nvm.stdout_is_tty, false);
        let out = colors::wrap_with_color_code(nvm, letter, text);
        nvm.stdout_is_tty = tty;
        out
    };
    let indent = " ".repeat(48);
    nvm.io
        .echo(&format!("{}Initial colors are:", " ".repeat(47)));
    let initial: String = ["b", "y", "g", "r", "e"]
        .iter()
        .map(|letter| wrap(nvm, letter, letter))
        .collect();
    nvm.io.echo(&format!("{}  {initial}", " ".repeat(48)));
    nvm.io.echo(&format!("{}Color codes:", " ".repeat(47)));
    for (low, high, low_name, high_name) in [
        ("r", "R", "red", "bold red"),
        ("g", "G", "green", "bold green"),
        ("b", "B", "blue", "bold blue"),
        ("c", "C", "cyan", "bold cyan"),
        ("m", "M", "magenta", "bold magenta"),
        ("y", "Y", "yellow", "bold yellow"),
        ("k", "K", "black", "bold black"),
        ("e", "W", "light grey", "white"),
    ] {
        let line = format!(
            "{indent}{}/{} = {} / {}",
            wrap(nvm, low, low),
            wrap(nvm, high, high),
            wrap(nvm, low, low_name),
            wrap(nvm, high, high_name)
        );
        nvm.io.echo(&colors::printf_b(&line));
    }
    nvm.restore("NVM_HAS_COLORS", saved);
}

/// nvm: nvm:set-colors
fn set_colors(nvm: &mut Nvm, args: &[String]) -> i32 {
    let code = args.first().map_or("", String::as_str);
    if set_colors_to(nvm, code) == 17 {
        nvm.to_stderr(|nvm| print_help(nvm, &["--help".to_string()]));
        nvm.io.echo("");
        nvm.io.err(&colors::printf_b(messages::INVALID_COLOR_CODES));
    }
    0
}

/// Sets the five colors `nvm ls` uses, from five letters. Returns 17 when
/// the letters are not five valid color codes.
///
/// nvm: nvm_set_colors
pub fn set_colors_to(nvm: &mut Nvm, code: &str) -> i32 {
    let valid = code.chars().count() == 5 && code.chars().all(|c| "rRgGbBcCyYmMkKeW".contains(c));
    if !valid {
        return 17;
    }
    let letters: Vec<String> = code.chars().map(String::from).collect();
    if !colors::has_colors(nvm) {
        nvm.io.echo(&messages::setting_colors(&letters.join(" ")));
        nvm.io.echo(messages::COLORS_NOT_SUPPORTED);
    } else {
        let saved = nvm.local("NVM_HAS_COLORS");
        nvm.vars.set_quiet("NVM_HAS_COLORS", "1");
        let painted: String = letters
            .iter()
            .map(|letter| colors::wrap_with_color_code(nvm, letter, letter))
            .collect();
        nvm.restore("NVM_HAS_COLORS", saved);
        nvm.io
            .echo(&colors::printf_b(&messages::setting_colors(&painted)));
    }
    nvm.vars.export("NVM_COLORS", code);
    0
}
