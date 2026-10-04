//! The install lock: a directory per version under the download cache, so
//! two `nvm install` runs of the same version do not work on its directory
//! at the same time. Making a directory either works or fails as one step,
//! which is what makes it a lock.

use super::tools;
use crate::cmd::cache::cache_dir;
use crate::context::Nvm;
use crate::text::install as text;
use std::fs;
use std::time::Duration;

/// A version as a file name: every byte outside `[0-9A-Za-z._+-]` becomes
/// an underscore.
///
/// nvm: nvm_install_lock_name
pub fn lock_name(version: &str) -> String {
    version
        .bytes()
        .map(|byte| match byte {
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'.' | b'_' | b'+' | b'-' => byte as char,
            _ => '_',
        })
        .collect()
}

/// `find <lock> -maxdepth 0 -type d -mmin +<minutes>`: whether the lock is
/// a directory last changed more than that many minutes ago. find counts
/// started minutes, so 61 seconds is two.
fn older_than(lock: &str, minutes: &str) -> bool {
    let Ok(minutes) = minutes.trim().parse::<u64>() else {
        return false;
    };
    let Ok(meta) = fs::symlink_metadata(lock) else {
        return false;
    };
    let age = meta
        .modified()
        .ok()
        .and_then(|time| time.elapsed().ok())
        .unwrap_or(Duration::ZERO);
    meta.is_dir() && age.as_secs().div_ceil(60) > minutes
}

/// Takes the lock for a version, waiting while another install holds it.
/// `NVM_INSTALL_LOCK_TIMEOUT` is how many seconds to wait, 600 unless set.
/// `NVM_INSTALL_LOCK_STALE` is the age in minutes after which a lock counts
/// as left behind by a crash and is taken over, never unless set. Returns 1
/// when the wait ran out.
///
/// nvm: nvm_acquire_install_lock
pub fn acquire(nvm: &mut Nvm, version: &str) -> i32 {
    if version.is_empty() {
        return 0;
    }
    let root = format!("{}/locks", cache_dir(nvm));
    // A lock directory that cannot be made does not stop the install.
    if fs::create_dir_all(&root).is_err() {
        return 0;
    }
    let lock = format!("{root}/{}", lock_name(version));
    let or = |value: &str, default: &str| match value {
        "" => default.to_string(),
        value => value.to_string(),
    };
    let timeout = or(nvm.vars.get("NVM_INSTALL_LOCK_TIMEOUT"), "600");
    let stale = or(nvm.vars.get("NVM_INSTALL_LOCK_STALE"), "0");

    let mut waited: u64 = 0;
    let mut announced = false;
    while fs::create_dir(&lock).is_err() {
        if stale != "0" && older_than(&lock, &stale) {
            nvm.io.err(&text::removing_stale_lock(version, &stale));
            tools::rm_rf_quiet(&[&lock]);
            continue;
        }
        if timeout
            .trim()
            .parse::<u64>()
            .is_ok_and(|limit| waited >= limit)
        {
            nvm.io.err(&text::lock_timed_out(&timeout, version));
            nvm.io.err(&text::lock_remove_hint(&lock));
            return 1;
        }
        if !announced {
            nvm.io.err(&text::lock_waiting(version));
            announced = true;
        }
        nvm.io.flush();
        std::thread::sleep(Duration::from_secs(1));
        waited += 1;
    }
    nvm.vars.assign("NVM_INSTALL_LOCK", lock);
    0
}

/// Gives back the lock [`acquire`] took, if there is one.
///
/// nvm: nvm_release_install_lock
pub fn release(nvm: &mut Nvm) {
    let lock = nvm.vars.get("NVM_INSTALL_LOCK").to_string();
    if !lock.is_empty() {
        if fs::remove_dir(&lock).is_err() {
            tools::rm_rf_quiet(&[&lock]);
        }
        nvm.vars.unset("NVM_INSTALL_LOCK");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_names_keep_only_safe_characters() {
        assert_eq!(lock_name("v22.20.0"), "v22.20.0");
        assert_eq!(lock_name("iojs-v3.3.1"), "iojs-v3.3.1");
        assert_eq!(lock_name("../x y"), ".._x_y");
        // `tr` works on bytes, so a two-byte character becomes two.
        assert_eq!(lock_name("é"), "__");
    }
}
