//! nvmimic's side of a sandbox: its scripts in `$NVM_DIR`, and its own files
//! in a folder beside the sandbox's home.
//!
//! An installed nvmimic keeps its own files under the home folder (see
//! `nvmimic::dirs`). A sandbox keeps them in `own/` next to `home/` instead,
//! so the comparison of the two home folders only ever sees what nvm's
//! commands wrote. `nvm.sh` names every one of these files by its full path
//! either way.

use crate::harness::{Outcome, copy, link_staged, nvmimic_bin, repo_root, write_script};
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
        binary: &bin.join("nvmimic"),
        nvm_functions: &own.join("nvm-functions.sh"),
        nvmimic_functions: &own.join("nvmimic-functions.sh"),
        cache: Some(&cache),
    };
    let template = fs::read_to_string(shell_dir.join("nvm.sh")).unwrap();
    fs::write(nvm_dir.join("nvm.sh"), bind(&template, &bound)).unwrap();
    for name in ["nvm-functions.sh", "nvmimic-functions.sh"] {
        copy(&shell_dir.join(name), &own.join(name));
    }

    let binary = nvmimic_bin();
    if count_launches {
        // A wrapper in the program's place notes each start and runs it.
        write_script(
            &bin.join("nvmimic"),
            &format!(
                "#!/bin/sh\n[ -z \"$NVMIMIC_LAUNCH_LOG\" ] || printf 'nvmimic\\n' >> \"$NVMIMIC_LAUNCH_LOG\"\nexec '{}' \"$@\"\n",
                binary.display()
            ),
        );
    } else {
        std::os::unix::fs::symlink(&binary, bin.join("nvmimic")).unwrap();
    }
}

/// nvmimic's one shell function that nvm does not have, see `shell/nvm.sh`.
const HANDOFF: &str = "__nvmimic_handoff";

/// Takes nvmimic's own function out of the list of functions a run left
/// behind. Every other name in that list is compared with real nvm's.
pub fn hide_own_function(outcome: &mut Outcome) {
    if let Some(functions) = outcome.state.get_mut("functions") {
        *functions = functions
            .split(' ')
            .filter(|name| *name != HANDOFF)
            .collect::<Vec<_>>()
            .join(" ");
    }
}
