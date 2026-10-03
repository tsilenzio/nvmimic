//! The nvm release this build matches, taken from `nvm-pin.toml` at build time.

/// nvm's version without the leading `v`, as `nvm --version` prints it.
pub const VERSION: &str = env!("NVM_PIN_VERSION");

/// The git tag of the pinned release.
pub const TAG: &str = env!("NVM_PIN_TAG");

/// The full commit hash the tag points at.
pub const COMMIT: &str = env!("NVM_PIN_COMMIT");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_matches_version() {
        assert_eq!(TAG, format!("v{VERSION}"));
        assert_eq!(COMMIT.len(), 40);
    }
}
