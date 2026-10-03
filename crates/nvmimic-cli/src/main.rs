use std::env;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args_os();
    let argv0 = args.next().unwrap_or_default();
    let as_nvm = Path::new(&argv0)
        .file_name()
        .is_some_and(|name| name == "nvm");
    let args: Vec<_> = args.collect();

    if !as_nvm && args.first().is_some_and(|a| a == "--version") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    if args.first().is_some_and(|a| a == "--version" || a == "-v") {
        println!("{}", nvmimic::pin::VERSION);
        return ExitCode::SUCCESS;
    }
    eprintln!("nvmimic: not implemented yet");
    ExitCode::from(127)
}
