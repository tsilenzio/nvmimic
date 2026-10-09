//! nvmimic's side of a sandbox: its scripts in `$NVM_DIR`, and its own files
//! in a folder beside the sandbox's home.
//!
//! An installed nvmimic keeps its own files under the home folder (see
//! `nvmimic::dirs`). A sandbox keeps them in `own/` next to `home/` instead,
//! so the comparison of the two home folders only ever sees what nvm's
//! commands wrote. `nvm.sh` names every one of these files by its full path
//! either way.

use crate::harness::{copy, link_staged, nvmimic_bin, repo_root, write_script};
use crate::subject::Subject;
use nvmimic::identity::Identity;
use nvmimic::install::nvm_sh;
use nvmimic::shell::quote;
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
    // An installed nvmimic finds its cache folder under the home folder.
    // Here it is told where, so the home folder stays as nvm leaves it.
    let program = quote(&bin.join("nvmimic").display().to_string());
    let init = format!("__NVMIMIC_PROGRAM={program} command {program} init");
    let script = nvm_sh(&Identity::NVMIMIC, &bin.join("nvmimic"));
    assert!(
        script.contains(&init),
        "nvm.sh no longer runs `init` this way"
    );
    // `init` prints the switch to the default version behind a check of
    // what it saw. With `NVMIMIC_PARITY_SECOND_START` set it is shown a
    // variable the shell doesn't hold, so the check fails in every shell
    // and loading takes its other way, a second start of the program.
    let second_start = match std::env::var_os("NVMIMIC_PARITY_SECOND_START") {
        Some(_) => "CC=nvmimic-parity-second-start ",
        None => "",
    };
    let told = format!(
        "NVMIMIC_CACHE_DIR={} {second_start}{init}",
        quote(&cache.display().to_string())
    );
    fs::write(nvm_dir.join("nvm.sh"), script.replace(&init, &told)).unwrap();

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

/// nvmimic's shell functions that nvm does not have: the hand-off, see
/// `shell/init.sh`, and under zsh the completion function that
/// `shell/bash_completion` registers.
const OWN_FUNCTIONS: [&str; 2] = ["__nvmimic_handoff", "_nvmimic_complete"];

/// nvmimic as the program a run compares with official nvm.
pub struct Nvmimic;

impl Subject for Nvmimic {
    fn name(&self) -> &str {
        "nvmimic"
    }

    fn install(&self, nvm_dir: &Path, side: &Path, count_launches: bool) {
        install(nvm_dir, side, count_launches)
    }

    fn own_functions(&self) -> Vec<String> {
        OWN_FUNCTIONS.map(str::to_string).to_vec()
    }
}
