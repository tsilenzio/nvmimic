//! POSIX regular expressions, as grep, sed and awk read the patterns nvm
//! builds. nvm splices `$NVM_DIR` and user input straight into them, so a
//! path holding regex characters has to match the way those tools match it.

use regex::Regex;

/// A basic regular expression (grep, sed without `-E`). `None` when the
/// pattern uses something with no equivalent, such as a back-reference, or
/// would make the tool fail.
pub fn bre(pattern: &str) -> Option<Regex> {
    Regex::new(&translate(pattern, false)?).ok()
}

/// An extended regular expression (grep -E, sed -E, awk).
pub fn ere(pattern: &str) -> Option<Regex> {
    Regex::new(&translate(pattern, true)?).ok()
}

fn translate(pattern: &str, extended: bool) -> Option<String> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    // Whether a `*` here would have nothing to repeat, which makes it literal in a BRE.
    let mut at_start = true;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '\\' => {
                let Some(&next) = chars.get(i + 1) else {
                    out.push_str(r"\\");
                    i += 1;
                    continue;
                };
                i += 2;
                if !extended {
                    match next {
                        '(' => {
                            out.push('(');
                            at_start = true;
                            continue;
                        }
                        ')' => out.push(')'),
                        '{' => out.push('{'),
                        '}' => out.push('}'),
                        '1'..='9' => return None,
                        _ => out.push_str(&regex::escape(&next.to_string())),
                    }
                } else {
                    match next {
                        '1'..='9' => return None,
                        _ => out.push_str(&regex::escape(&next.to_string())),
                    }
                }
                at_start = false;
                continue;
            }
            '[' => {
                let (class, used) = bracket(&chars[i..])?;
                out.push_str(&class);
                i += used;
                at_start = false;
                continue;
            }
            '*' if at_start => out.push_str(r"\*"),
            '^' if !extended && !at_start => out.push_str(r"\^"),
            '$' if !extended
                && i + 1 != chars.len()
                && !(chars.get(i + 1) == Some(&'\\') && chars.get(i + 2) == Some(&')')) =>
            {
                out.push_str(r"\$")
            }
            '+' | '?' | '|' | '(' | ')' | '{' | '}' if !extended => {
                out.push_str(&regex::escape(&c.to_string()))
            }
            '(' if extended => {
                out.push('(');
                i += 1;
                at_start = true;
                continue;
            }
            '{' if extended && !is_interval(&chars[i..]) => out.push_str(r"\{"),
            '}' if extended => out.push('}'),
            _ => out.push(c),
        }
        at_start = c == '^' && out.ends_with('^');
        i += 1;
    }
    Some(out)
}

fn is_interval(rest: &[char]) -> bool {
    let close = rest.iter().position(|&c| c == '}');
    close.is_some_and(|end| end > 1 && rest[1..end].iter().all(|c| c.is_ascii_digit() || *c == ','))
}

/// A bracket expression starting at `rest[0] == '['`. Returns the Rust class
/// and how many characters it used.
fn bracket(rest: &[char]) -> Option<(String, usize)> {
    let mut out = String::from("[");
    let mut i = 1;
    if rest.get(i) == Some(&'^') {
        out.push('^');
        i += 1;
    }
    // A `]` right after the opening (or after `^`) is a literal member.
    if rest.get(i) == Some(&']') {
        out.push_str(r"\]");
        i += 1;
    }
    while i < rest.len() {
        let c = rest[i];
        match c {
            ']' => {
                out.push(']');
                return Some((out, i + 1));
            }
            '[' if matches!(rest.get(i + 1), Some(':' | '.' | '=')) => {
                let kind = rest[i + 1];
                let end = (i + 2..rest.len().saturating_sub(1))
                    .find(|&j| rest[j] == kind && rest[j + 1] == ']')?;
                let name: String = rest[i + 2..end].iter().collect();
                if kind == ':' {
                    out.push_str(&format!("[:{name}:]"));
                } else {
                    out.push_str(&regex::escape(&name));
                }
                i = end + 2;
                continue;
            }
            '\\' | '[' | '&' | '~' => {
                out.push('\\');
                out.push(c);
            }
            '-' if rest.get(i + 1) == Some(&'-') => out.push_str(r"\-"),
            _ => out.push(c),
        }
        i += 1;
    }
    None
}

/// Expands a sed replacement: `&` is the whole match, `\n` (a digit) a group,
/// and a backslash before anything else makes it literal.
pub fn sed_replacement(template: &str, caps: &regex::Captures) -> String {
    let mut out = String::new();
    let mut chars = template.chars();
    while let Some(c) = chars.next() {
        match c {
            '&' => out.push_str(&caps[0]),
            '\\' => match chars.next() {
                Some(d @ '0'..='9') => {
                    let index = d.to_digit(10).unwrap() as usize;
                    out.push_str(caps.get(index).map_or("", |m| m.as_str()));
                }
                Some(other) => out.push(other),
                None => {}
            },
            _ => out.push(c),
        }
    }
    out
}

/// sed's `s/pattern/replacement/` on one line, replacing the first match.
pub fn sed_first(line: &str, pattern: &Regex, replacement: &str) -> String {
    match pattern.captures(line) {
        Some(caps) => {
            let whole = caps.get(0).unwrap();
            let mut out = String::with_capacity(line.len());
            out.push_str(&line[..whole.start()]);
            out.push_str(&sed_replacement(replacement, &caps));
            out.push_str(&line[whole.end()..]);
            out
        }
        None => line.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bre_treats_ere_operators_as_literals() {
        let re = bre("a+b").unwrap();
        assert!(re.is_match("xa+by"));
        assert!(!re.is_match("aab"));
        assert!(bre(r"\(ab\)*").unwrap().is_match("abab"));
    }

    #[test]
    fn brackets_and_classes() {
        let re = bre("/x/[^/]*/bin").unwrap();
        assert!(re.is_match("/x/v1/bin"));
        assert!(!re.is_match("/x/a/b/bin"));
        assert!(ere("^[[:space:]]*$").unwrap().is_match("  "));
        assert!(bre("[]a]").unwrap().is_match("]"));
    }

    #[test]
    fn dots_in_paths_match_anything() {
        assert!(bre("/home/.nvm/").unwrap().is_match("/home/xnvm/"));
    }

    #[test]
    fn replacement_expands_ampersand() {
        let re = bre("b").unwrap();
        assert_eq!(sed_first("abc", &re, "[&]"), "a[b]c");
        assert_eq!(sed_first("abc", &re, r"\&"), "a&c");
    }
}
