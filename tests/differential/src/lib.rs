//! The parity harness: official nvm and nvmimic run the same shell snippets in
//! isolated homes that share one fake Node mirror, and everything they print,
//! return, export, define or write is compared.

pub mod cases;
pub mod covers;
pub mod golden;
pub mod guard;
pub mod harness;
pub mod mirror;
pub mod nvmsh;
pub mod own;
pub mod subject;
pub mod suite;

use std::path::PathBuf;
use std::sync::OnceLock;

/// The vendored copy of the nvm release nvmimic is pinned to.
pub fn reference_dir() -> PathBuf {
    harness::repo_root().join(format!("tests/reference/nvm-{}", nvmimic::pin::TAG))
}

/// Every function the pinned `nvm.sh` defines, plus `nvm` itself. The probe
/// checks each one by name, which works in shells that cannot list functions.
pub fn nvm_function_names() -> &'static [String] {
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let source = std::fs::read_to_string(reference_dir().join("nvm.sh")).unwrap();
        let mut names: Vec<String> = source
            .lines()
            .filter_map(|line| line.strip_suffix("() {"))
            .filter(|name| name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
            .map(str::to_string)
            .collect();
        names.sort();
        names.dedup();
        names
    })
}
