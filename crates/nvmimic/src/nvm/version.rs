//! Version strings: comparison, normalization, prefixes and validity, as
//! nvm's version helpers compute them.

use super::awk::{self, Field};

pub const IOJS_PREFIX: &str = "iojs";
pub const NODE_PREFIX: &str = "node";

/// The awk program nvm uses for "is `a` greater than `b`", as its exit
/// status: 0 for greater, 1 for a missing argument, 2 for a non-numeric `a`,
/// 3 for less, 4 for equal. A non-numeric field in `b` counts as greater.
///
/// nvm: nvm_version_greater
pub fn greater_status(a: &str, b: &str) -> i32 {
    let a = a.strip_prefix('v').unwrap_or(a);
    let b = b.strip_prefix('v').unwrap_or(b);
    if a.is_empty() || b.is_empty() {
        return 1;
    }
    let (a, b) = (awk::split_dots(a), awk::split_dots(b));
    for i in 0..3 {
        let (x, y) = (Field::of(&a, i), Field::of(&b, i));
        if x.truthy() && !is_digits(x) {
            return 2;
        }
        if y.truthy() && !is_digits(y) {
            return 0;
        }
        if awk::less(x, y) {
            return 3;
        } else if awk::greater(x, y) {
            return 0;
        }
    }
    4
}

/// nvm: nvm_version_greater
pub fn greater(a: &str, b: &str) -> bool {
    greater_status(a, b) == 0
}

/// Exit status of nvm's "is `a` at least `b`": 0 for true, 1 for a missing
/// argument, 2 for a non-numeric `a`, 3 for less.
///
/// nvm: nvm_version_greater_than_or_equal_to
pub fn gte_status(a: &str, b: &str) -> i32 {
    let a = a.strip_prefix('v').unwrap_or(a);
    let b = b.strip_prefix('v').unwrap_or(b);
    if a.is_empty() || b.is_empty() {
        return 1;
    }
    let (a, b) = (awk::split_dots(a), awk::split_dots(b));
    for i in 0..3 {
        let (x, y) = (Field::of(&a, i), Field::of(&b, i));
        if x.truthy() && !is_digits(x) {
            return 2;
        }
        if awk::less(x, y) {
            return 3;
        } else if awk::greater(x, y) {
            return 0;
        }
    }
    0
}

/// nvm: nvm_version_greater_than_or_equal_to
pub fn gte(a: &str, b: &str) -> bool {
    gte_status(a, b) == 0
}

fn is_digits(field: Field) -> bool {
    match field {
        Field::Present(s) => !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()),
        Field::Missing => false,
    }
}

/// `major * 10^12 + minor * 10^6 + patch`, printed the way awk's
/// `%d%06d%06d` prints it.
///
/// nvm: nvm_normalize_version
pub fn normalize(version: &str) -> String {
    let version = version.strip_prefix('v').unwrap_or(version);
    let parts = awk::split_dots(version);
    let field = |i: usize| parts.get(i).map_or(0, |s| awk::to_number(s) as i64);
    format!("{}{:06}{:06}", field(0), field(1), field(2))
}

/// How many dot-separated groups a version has, ignoring a leading `v` and
/// one trailing dot.
///
/// nvm: nvm_num_version_groups
pub fn num_groups(version: &str) -> usize {
    let version = version.strip_prefix('v').unwrap_or(version);
    let version = version.strip_suffix('.').unwrap_or(version);
    if version.is_empty() {
        return 0;
    }
    version.matches('.').count() + 1
}

/// nvm: nvm_is_iojs_version
pub fn is_iojs(version: &str) -> bool {
    version.starts_with("iojs-")
}

/// nvm: nvm_strip_iojs_prefix
pub fn strip_iojs_prefix(version: &str) -> &str {
    if version == IOJS_PREFIX {
        ""
    } else {
        version.strip_prefix("iojs-").unwrap_or(version)
    }
}

/// nvm: nvm_add_iojs_prefix
pub fn add_iojs_prefix(version: &str) -> String {
    format!(
        "{IOJS_PREFIX}-{}",
        ensure_prefix(strip_iojs_prefix(version))
    )
}

/// Puts a `v` in front of a version that starts with a digit, keeping an
/// `iojs-` prefix in place.
///
/// nvm: nvm_ensure_version_prefix
pub fn ensure_prefix(version: &str) -> String {
    let stripped = strip_iojs_prefix(version);
    let prefixed = if stripped.starts_with(|c: char| c.is_ascii_digit()) {
        format!("v{stripped}")
    } else {
        stripped.to_string()
    };
    if is_iojs(version) {
        add_iojs_prefix(&prefixed)
    } else {
        prefixed
    }
}

/// Pads a version to three groups and cuts anything past the third.
///
/// nvm: nvm_format_version
pub fn format(version: &str) -> String {
    let version = ensure_prefix(version);
    if num_groups(&version) < 3 {
        let trimmed = version.strip_suffix('.').unwrap_or(&version);
        format(&format!("{trimmed}.0"))
    } else {
        version.split('.').take(3).collect::<Vec<_>>().join(".")
    }
}

/// nvm: nvm_validate_implicit_alias
pub fn is_implicit_alias(name: &str) -> bool {
    matches!(name, "stable" | "unstable" | IOJS_PREFIX | NODE_PREFIX)
}

/// nvm: nvm_is_valid_version
pub fn is_valid(version: &str) -> bool {
    if is_implicit_alias(version) {
        return true;
    }
    gte(strip_iojs_prefix(version), "0")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greater_matches_nvm_exit_codes() {
        assert_eq!(greater_status("0.12.0", "v0.10.1"), 0);
        assert_eq!(greater_status("0.12.0", "v22.1.0"), 3);
        assert_eq!(greater_status("v1.2.3", "1.2.3"), 4);
        assert_eq!(greater_status("", "1"), 1);
        assert_eq!(greater_status("x", "1"), 2);
        assert_eq!(greater_status("1", "x"), 0);
    }

    #[test]
    fn validity() {
        assert!(is_valid("22"));
        assert!(is_valid("v22.1.0"));
        assert!(is_valid("node"));
        assert!(is_valid("iojs-v3.3.1"));
        assert!(!is_valid("lts/*"));
        assert!(!is_valid("default"));
        // awk decides on the first field, so a bad later field never counts.
        assert!(is_valid("22.x"));
        assert!(!is_valid(""));
    }

    #[test]
    fn prefixes_and_formatting() {
        assert_eq!(ensure_prefix("22.1"), "v22.1");
        assert_eq!(ensure_prefix("iojs-3.3.1"), "iojs-v3.3.1");
        assert_eq!(ensure_prefix("lts/*"), "lts/*");
        assert_eq!(format("22"), "v22.0.0");
        assert_eq!(format("v22.1.2.3"), "v22.1.2");
        assert_eq!(num_groups("v22."), 1);
        assert_eq!(normalize("v22.1.0"), "22000001000000");
        assert_eq!(normalize("0.10"), "0000010000000");
    }
}
