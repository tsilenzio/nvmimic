//! `nvm cache` and `nvm clear-cache`.

use crate::context::Nvm;
use crate::nvm::tools;
use crate::text::install as text;
use std::fs;

/// Where downloads are kept.
///
/// nvm: nvm_cache_dir
pub fn cache_dir(nvm: &Nvm) -> String {
    format!("{}/.cache", nvm.vars.get("NVM_DIR"))
}

/// nvm: nvm:cache
pub fn run(nvm: &mut Nvm, args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("dir") => {
            let dir = cache_dir(nvm);
            nvm.io.echo(&dir);
            0
        }
        Some("clear") => {
            let dir = cache_dir(nvm);
            // nvm runs `rm -rf` and `mkdir -p`, and what `rm` says when it
            // fails is part of nvm's output, so the same two programs run here.
            let ran = |nvm: &mut Nvm, program: &str, flag: &str| {
                tools::run(nvm, program, &[flag, &dir]).is_some_and(|ran| ran.ok())
            };
            if ran(nvm, "rm", "-rf") && ran(nvm, "mkdir", "-p") {
                nvm.io.echo(text::CACHE_CLEARED);
                0
            } else {
                nvm.io.err(&text::cache_clear_failed(&dir));
                1
            }
        }
        _ => {
            for line in text::CACHE_USAGE {
                nvm.io.err(line);
            }
            127
        }
    }
}

/// nvm removes two paths here without `-r`: a file literally named `v*` in
/// `$NVM_DIR` (the name is quoted, so it never expands) and the versions
/// directory, which `rm -f` refuses because it is a directory. In practice
/// nothing is deleted and the message is all that happens.
///
/// nvm: nvm:clear-cache
pub fn clear_cache(nvm: &mut Nvm) -> i32 {
    let root = nvm.vars.get("NVM_DIR").to_string();
    for path in [format!("{root}/v*"), format!("{root}/versions/node")] {
        if fs::symlink_metadata(&path).is_ok_and(|meta| !meta.is_dir()) {
            let _ = fs::remove_file(&path);
        }
    }
    nvm.io.echo(text::CACHE_CLEARED);
    0
}
