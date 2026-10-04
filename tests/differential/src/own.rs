//! nvmimic's side of a sandbox: its scripts in `$NVM_DIR`, and its own files
//! in a folder beside the sandbox's home.
//!
//! An installed nvmimic keeps its own files under the home folder (see
//! `nvmimic::dirs`). A sandbox keeps them in `own/` next to `home/` instead,
//! so the comparison of the two home folders only ever sees what nvm's
//! commands wrote. `nvm.sh` names every one of these files by its full path
//! either way.

use crate::harness::{copy, link_staged, nvmimic_bin, repo_root, write_script};
use nvmimic::install::{Bound, bind};
use std::fs;
use std::path::{Path, PathBuf};

/// Where a sandbox keeps nvmimic's own files.
pub fn dir(side: &Path) -> PathBuf {
    side.join("own")
}

/// Puts nvmimic into `$NVM_DIR` the way `nvmimic self install` would.
pub fn install(nvm_dir: &Path, side: &Path, count_launches: bool) {
    let shell_dir = repo_root().join("shell");
    copy(
        &shell_dir.join("bash_completion"),
        &nvm_dir.join("bash_completion"),
    );
    link_staged("nvmimic/nvm-exec", &shell_dir.join("nvm-exec"), nvm_dir);

    let own = dir(side);
    let bin = own.join("bin");
    let cache = own.join("cache");
    for dir in [&bin, &cache] {
        fs::create_dir_all(dir).unwrap();
    }
    let bound = Bound {
        binary: &bin.join("nvm"),
        nvm_functions: &own.join("nvm-functions.sh"),
        nvmimic_functions: &own.join("nvmimic-functions.sh"),
        cache: Some(&cache),
    };
    for (name, to) in [
        ("nvm.sh", nvm_dir.join("nvm.sh")),
        ("nvmimic-functions.sh", own.join("nvmimic-functions.sh")),
    ] {
        let template = fs::read_to_string(shell_dir.join(name)).unwrap();
        fs::write(to, bind(&template, &bound)).unwrap();
    }
    copy(
        &shell_dir.join("nvm-functions.sh"),
        &own.join("nvm-functions.sh"),
    );

    let binary = nvmimic_bin();
    std::os::unix::fs::symlink(&binary, bin.join("nvmimic")).unwrap();
    if count_launches {
        // The binary picks its persona from argv0, so the logging wrapper
        // execs a symlink that is also named `nvm`.
        let real = side.join("real");
        fs::create_dir_all(&real).unwrap();
        std::os::unix::fs::symlink(&binary, real.join("nvm")).unwrap();
        write_script(
            &bin.join("nvm"),
            &format!(
                "#!/bin/sh\n[ -z \"$NVMIMIC_LAUNCH_LOG\" ] || printf 'nvmimic\\n' >> \"$NVMIMIC_LAUNCH_LOG\"\nexec '{}' \"$@\"\n",
                real.join("nvm").display()
            ),
        );
    } else {
        std::os::unix::fs::symlink("nvmimic", bin.join("nvm")).unwrap();
    }
}
