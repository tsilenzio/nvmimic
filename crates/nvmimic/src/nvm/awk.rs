//! The small parts of awk's value rules that nvm's one-line awk programs
//! depend on. nvm compares version fields with awk, so a field such as `"0"`
//! is false, `"10"` compares as a number and `"x"` compares as a string.

/// A field from awk's `split()`: present with its text, or past the end.
#[derive(Clone, Copy, Debug)]
pub enum Field<'a> {
    Present(&'a str),
    Missing,
}

impl<'a> Field<'a> {
    pub fn of(parts: &[&'a str], index: usize) -> Field<'a> {
        parts
            .get(index)
            .map_or(Field::Missing, |s| Field::Present(s))
    }

    /// awk's truth test on a field: an uninitialized value is false, a value
    /// that looks numeric is false when it equals zero, any other string is
    /// false only when empty.
    pub fn truthy(self) -> bool {
        match self {
            Field::Missing => false,
            Field::Present(s) => match looks_numeric(s) {
                Some(value) => value != 0.0,
                None => !s.is_empty(),
            },
        }
    }

    fn text(self) -> &'a str {
        match self {
            Field::Present(s) => s,
            Field::Missing => "",
        }
    }

    fn numeric(self) -> Option<f64> {
        match self {
            Field::Missing => Some(0.0),
            Field::Present(s) => looks_numeric(s),
        }
    }
}

/// awk's `a < b` for two fields: numeric when both look like numbers (an
/// uninitialized field counts as one), otherwise a string comparison.
pub fn less(a: Field, b: Field) -> bool {
    match (a.numeric(), b.numeric()) {
        (Some(x), Some(y)) => x < y,
        _ => a.text() < b.text(),
    }
}

pub fn greater(a: Field, b: Field) -> bool {
    less(b, a)
}

/// Whether awk treats input text as a number: optional blanks, a sign, a
/// decimal number with an optional exponent, optional blanks, nothing else.
pub fn looks_numeric(s: &str) -> Option<f64> {
    let t = s.trim_matches([' ', '\t', '\n']);
    if t.is_empty() {
        return None;
    }
    let bytes = t.as_bytes();
    let mut i = 0;
    if bytes[i] == b'+' || bytes[i] == b'-' {
        i += 1;
    }
    let digits_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let mut digits = i - digits_start;
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let fraction_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        digits += i - fraction_start;
    }
    if digits == 0 {
        return None;
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        let mut j = i + 1;
        if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
            j += 1;
        }
        let exponent_start = j;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if j > exponent_start {
            i = j;
        }
    }
    if i != bytes.len() {
        return None;
    }
    t.parse().ok()
}

/// awk's conversion of a string to a number: the longest numeric prefix, or 0.
pub fn to_number(s: &str) -> f64 {
    let t = s.trim_start_matches([' ', '\t', '\n']);
    let bytes = t.as_bytes();
    let mut end = 0;
    let mut best = 0;
    if end < bytes.len() && (bytes[end] == b'+' || bytes[end] == b'-') {
        end += 1;
    }
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
        best = end;
    }
    if end < bytes.len() && bytes[end] == b'.' {
        end += 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
            best = end;
        }
    }
    t[..best].parse().unwrap_or(0.0)
}

/// `split(s, a, /\./)`: awk splits an empty string into no fields at all.
pub fn split_dots(s: &str) -> Vec<&str> {
    if s.is_empty() {
        Vec::new()
    } else {
        s.split('.').collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_detection_follows_awk() {
        assert_eq!(looks_numeric("10"), Some(10.0));
        assert_eq!(looks_numeric(" 1.5 "), Some(1.5));
        assert_eq!(looks_numeric("1e3"), Some(1000.0));
        assert_eq!(looks_numeric("x"), None);
        assert_eq!(looks_numeric("1x"), None);
        assert_eq!(looks_numeric(""), None);
    }

    #[test]
    fn zero_is_false_and_strings_compare_as_text() {
        assert!(!Field::Present("0").truthy());
        assert!(Field::Present("x").truthy());
        assert!(less(Field::Present("9"), Field::Present("10")));
        assert!(less(Field::Present("10"), Field::Present("9x")));
        assert!(less(Field::Missing, Field::Present("1")));
    }

    #[test]
    fn numeric_prefix_conversion() {
        assert_eq!(to_number("22abc"), 22.0);
        assert_eq!(to_number("abc"), 0.0);
        assert_eq!(to_number("1.5.2"), 1.5);
    }
}
