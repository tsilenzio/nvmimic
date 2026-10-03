//! Reads one number from the terminfo database: how many colors a terminal
//! supports, which is what `tput -T "$TERM" colors` prints.

use std::fs;
use std::path::{Path, PathBuf};

const MAX_COLORS: usize = 13;

/// `None` when no terminfo entry exists for `term`, the case where tput
/// reports an unknown terminal. `Some(-1)` when the entry has no color count.
pub fn colors(term: &str, home: &str, terminfo: Option<&str>, dirs: Option<&str>) -> Option<i32> {
    if term.is_empty() || term.contains('/') {
        return None;
    }
    let first = term.chars().next()?;
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(dir) = terminfo.filter(|d| !d.is_empty()) {
        roots.push(dir.into());
    }
    if !home.is_empty() {
        roots.push(Path::new(home).join(".terminfo"));
    }
    if let Some(list) = dirs {
        for dir in list.split(':') {
            if dir.is_empty() {
                roots.push("/usr/share/terminfo".into());
            } else {
                roots.push(dir.into());
            }
        }
    }
    for dir in [
        "/usr/share/terminfo",
        "/etc/terminfo",
        "/lib/terminfo",
        "/usr/lib/terminfo",
    ] {
        roots.push(dir.into());
    }

    for root in roots {
        for sub in [first.to_string(), format!("{:x}", first as u32)] {
            if let Ok(bytes) = fs::read(root.join(&sub).join(term)) {
                return parse_colors(&bytes);
            }
        }
    }
    None
}

fn parse_colors(bytes: &[u8]) -> Option<i32> {
    let short = |at: usize| -> Option<i32> {
        let lo = *bytes.get(at)? as i32;
        let hi = *bytes.get(at + 1)? as i32;
        Some(((hi << 8) | lo) as i16 as i32)
    };
    let magic = short(0)? as u16;
    let number_size = match magic {
        0o432 => 2,
        0o1036 => 4,
        _ => return None,
    };
    let names_size = short(2)? as usize;
    let bool_count = short(4)? as usize;
    let num_count = short(6)? as usize;
    if MAX_COLORS >= num_count {
        return Some(-1);
    }
    let mut at = 12 + names_size + bool_count;
    if at % 2 == 1 {
        at += 1;
    }
    at += MAX_COLORS * number_size;
    let value = if number_size == 2 {
        short(at)?
    } else {
        i32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?)
    };
    Some(if value < 0 { -1 } else { value })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xterm_has_colors_when_installed() {
        if let Some(count) = colors("xterm-256color", "", None, None) {
            assert_eq!(count, 256);
        }
        assert_eq!(colors("no-such-terminal-here", "", None, None), None);
    }
}
