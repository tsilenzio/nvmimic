//! The `nvmimic` binary. People run it to manage their nvmimic installation.
//! `$NVM_DIR/nvm.sh` runs `nvmimic init` to get the shell functions, and the
//! functions run it through one internal command to do nvm's work.

use nvmimic::backend::NvmDirBackend;
use nvmimic::host::{HANDOFF, Host, is_tty};
use nvmimic::identity::Identity;
use nvmimic::init::Named;
use std::env;
use std::process::ExitCode;

/// nvmimic as a program built on its own library.
const HOST: Host = Host {
    identity: &Identity::NVMIMIC,
    backend: &NvmDirBackend,
    init_extra: "",
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    match args.split_first() {
        Some((first, rest)) if first == HANDOFF => match rest.split_first() {
            Some((function, rest)) => ExitCode::from(HOST.handoff(function, rest)),
            None => ExitCode::from(2),
        },
        _ => nvmimic_command(&args),
    }
}

/// `nvmimic init [shell]`: prints the shell code that loads nvmimic.
fn init(named: Option<Named>) -> ExitCode {
    match HOST.init(named) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("nvmimic: {message}");
            ExitCode::FAILURE
        }
    }
}

const INIT_USAGE: &str = "\
usage: nvmimic init [bash|zsh|sh|dash]

Prints the shell code that makes `nvm` work, for a shell to run with `eval`.
The nvm.sh that `nvmimic self install` writes does that for you. To do it
yourself, put this line in your startup file, with the name of your shell:

  eval \"$(nvmimic init zsh)\"

The README says what the name is for and which nvm folder gets used.";

const USAGE: &str = "\
usage: nvmimic <command>

nvmimic does nvm's job, only faster. In your shell you keep typing `nvm`.
This command sets nvmimic up, keeps it up to date and removes it again.

  self install     set nvmimic up for your nvm folder ($NVM_DIR, usually ~/.nvm)
  self eject       put official nvm's scripts back and keep nvmimic for later
  self uninstall   remove nvmimic again
  self update      get the newest nvmimic
  self rollback    go back to the nvmimic from before the last update
  init [shell]     print the shell code that loads nvmimic, for `eval`
  --version        print nvmimic's own version

Run nvmimic <command> --help for details.";

const SELF_USAGE: &str = "\
usage: nvmimic self <command> [--force] [--dry-run]

  install     set nvmimic up for your nvm folder ($NVM_DIR, usually ~/.nvm)
  eject       put official nvm's scripts back and keep nvmimic for later
  uninstall   remove nvmimic again
  update      get the newest nvmimic
  rollback    go back to the nvmimic from before the last update

  --force     don't ask, go ahead
  --dry-run   show what would happen and change nothing

Run nvmimic self <command> --help for details.";

const INSTALL_USAGE: &str = "\
usage: nvmimic self install [--force] [--dry-run]

Sets nvmimic up, so that `nvm` in your shell is nvmimic. If official nvm is
already in your nvm folder, its Node versions and aliases stay, and its three
scripts are kept so that `nvmimic self uninstall` can put them back.

  --force     don't ask, go ahead
  --dry-run   show what would happen and change nothing

NVM_DIR names the nvm folder if it isn't ~/.nvm. The README lists the
settings that move nvmimic's own files.";

const EJECT_USAGE: &str = "\
usage: nvmimic self eject [--force] [--dry-run]

Hands your nvm folder back to official nvm and keeps the nvmimic program, so
`nvmimic self install` can take over again later. It needs an official nvm
to go back to, which is one that was there before nvmimic.

  --force     don't ask, go ahead
  --dry-run   show what would happen and change nothing";

const UNINSTALL_USAGE: &str = "\
usage: nvmimic self uninstall [--purge] [--force] [--dry-run]

Removes nvmimic and puts official nvm's scripts back if nvmimic replaced
any. Your Node versions, aliases and the nvm folder itself stay.

  --purge     also delete the whole nvm folder, every Node version and
              alias in it. Official nvm is not put back
  --force     don't ask, go ahead
  --dry-run   show what would happen and change nothing";

const UPDATE_USAGE: &str = "\
usage: nvmimic self update [--force] [--dry-run]

Gets the newest nvmimic, checks the download and swaps it in. The nvmimic
you have now is kept, and `nvmimic self rollback` brings it back.

  --force     don't ask, go ahead
  --dry-run   show what would happen and change nothing";

const ROLLBACK_USAGE: &str = "\
usage: nvmimic self rollback [--force] [--dry-run]

Goes back to the nvmimic you had before the last `nvmimic self update`.
Running it twice gets you back where you started.

  --force     don't ask, go ahead
  --dry-run   show what would happen and change nothing";

/// The help for `nvmimic self <action>`, or the list of them for `self`
/// alone.
fn self_usage(action: Option<&str>) -> &'static str {
    match action {
        Some("install") => INSTALL_USAGE,
        Some("eject") => EJECT_USAGE,
        Some("uninstall") => UNINSTALL_USAGE,
        Some("update") => UPDATE_USAGE,
        Some("rollback") => ROLLBACK_USAGE,
        _ => SELF_USAGE,
    }
}

fn is_help(word: &str) -> bool {
    matches!(word, "help" | "--help" | "-h")
}

/// The commands people type.
fn nvmimic_command(args: &[String]) -> ExitCode {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    let (action, flags) = match words.as_slice() {
        [] => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        [word] if is_help(word) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        ["--version"] => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        ["init"] => return init(None),
        ["init", word] if is_help(word) => {
            println!("{INIT_USAGE}");
            return ExitCode::SUCCESS;
        }
        ["init", shell] => {
            return match Named::parse(shell) {
                Some(named) => init(Some(named)),
                None => {
                    eprintln!(
                        "nvmimic: init doesn't know the shell {shell}. It knows {}. Run nvmimic init --help.",
                        Named::NAMES.join(", ")
                    );
                    ExitCode::from(2)
                }
            };
        }
        ["self"] => {
            println!("{}", self_usage(None));
            return ExitCode::SUCCESS;
        }
        ["self", word] if is_help(word) => {
            println!("{}", self_usage(None));
            return ExitCode::SUCCESS;
        }
        [
            "self",
            action @ ("install" | "eject" | "uninstall" | "update" | "rollback"),
            flags @ ..,
        ] => (*action, flags),
        _ => {
            // Only `self` has commands of its own to point at.
            let help = match words.first() {
                Some(&"self") => "nvmimic self --help",
                _ => "nvmimic --help",
            };
            eprintln!(
                "nvmimic: there is no command {}. Run {help}.",
                words.join(" ")
            );
            return ExitCode::from(2);
        }
    };
    if flags.iter().any(|flag| is_help(flag)) {
        println!("{}", self_usage(Some(action)));
        return ExitCode::SUCCESS;
    }
    if let Some(unknown) = flags.iter().find(|flag| {
        !matches!(**flag, "--force" | "--dry-run")
            && !(action == "uninstall" && **flag == "--purge")
    }) {
        eprintln!(
            "nvmimic: self {action} has no option {unknown}. Run nvmimic self {action} --help."
        );
        return ExitCode::from(2);
    }
    let options = nvmimic::install::Options {
        force: flags.contains(&"--force"),
        dry_run: flags.contains(&"--dry-run"),
        purge: flags.contains(&"--purge"),
        terminal: is_tty(0),
    };
    // `self eject` and `self uninstall` say what they would have done
    // before they refuse, so they handle a missing terminal themselves.
    let leaving = matches!(action, "eject" | "uninstall");
    if !leaving && !options.force && !options.dry_run && !options.terminal {
        eprintln!(
            "nvmimic: this asks questions, and there is no terminal to answer them. Run it again with --force."
        );
        return ExitCode::from(2);
    }
    let env = nvmimic::install::Env::from_process();
    let mut ui = nvmimic::install::TerminalUi;
    let result = match action {
        "install" => match env::current_exe() {
            Ok(binary) => nvmimic::install::self_install(&env, &binary, &options, &mut ui),
            Err(error) => Err(format!("cannot find this program's own file: {error}")),
        },
        "update" => {
            let source = nvmimic::update::Source::from_process();
            nvmimic::update::self_update(&env, &source, &options, &mut ui)
        }
        "rollback" => nvmimic::update::self_rollback(&env, &options, &mut ui),
        "eject" => nvmimic::install::self_eject(&env, &options, &mut ui),
        _ => nvmimic::install::self_uninstall(&env, &options, &mut ui),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("nvmimic: {message}");
            match message == nvmimic::install::NO_TERMINAL {
                true => ExitCode::from(2),
                false => ExitCode::FAILURE,
            }
        }
    }
}
