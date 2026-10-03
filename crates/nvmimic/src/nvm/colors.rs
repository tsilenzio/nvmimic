//! Colored output: whether to color, which ANSI codes, and `set-colors`.

use super::{paths, terminfo};
use crate::context::Nvm;

/// The default for `NVM_COLORS`: installed, system, current, not installed, default.
pub const DEFAULT_COLORS: &str = "bygre";

/// Whether nvm colors its output here: stdout is a terminal, the terminal
/// has at least 8 colors, and `--no-colors` is not in effect.
///
/// nvm: nvm_has_colors
pub fn has_colors(nvm: &mut Nvm) -> bool {
    let mut count: Option<i32> = None;
    if paths::on_path(nvm.vars.get("PATH"), "tput") {
        let term = match nvm.vars.get("TERM") {
            "" => "vt100",
            term => term,
        }
        .to_string();
        count = terminfo::colors(
            &term,
            nvm.vars.get("HOME"),
            nvm.vars.get_opt("TERMINFO"),
            nvm.vars.get_opt("TERMINFO_DIRS"),
        );
        if count.is_none() {
            nvm.io.err(&format!("tput: unknown terminal \"{term}\""));
        }
    }
    nvm.stdout_is_tty && count.unwrap_or(-1) >= 8 && nvm.vars.get("NVM_NO_COLORS") != "--no-colors"
}

/// The ANSI code for one color letter, `None` for `0` (no color), or an
/// error message for anything else.
///
/// nvm: nvm_print_color_code
pub fn color_code(letter: &str) -> Result<Option<&'static str>, String> {
    Ok(Some(match letter {
        "0" => return Ok(None),
        "r" => "0;31m",
        "R" => "1;31m",
        "g" => "0;32m",
        "G" => "1;32m",
        "b" => "0;34m",
        "B" => "1;34m",
        "c" => "0;36m",
        "C" => "1;36m",
        "m" => "0;35m",
        "M" => "1;35m",
        "y" => "0;33m",
        "Y" => "1;33m",
        "k" => "0;30m",
        "K" => "1;30m",
        "e" => "0;37m",
        "W" => "1;37m",
        other => return Err(format!("Invalid color code: {other}")),
    }))
}

/// One of the six colors nvm uses, by index, from `NVM_COLORS`. Index 6 is
/// the LTS color: the system color with every `0` turned into `1`.
///
/// nvm: nvm_get_colors
pub fn get_color(nvm: &mut Nvm, index: usize) -> String {
    let colors = match nvm.vars.get("NVM_COLORS") {
        "" => DEFAULT_COLORS.to_string(),
        value => value.to_string(),
    };
    let letter = |i: usize| {
        colors
            .chars()
            .nth(i - 1)
            .map(String::from)
            .unwrap_or_default()
    };
    let code = |nvm: &mut Nvm, letter: String| match color_code(&letter) {
        Ok(code) => code.unwrap_or("").to_string(),
        Err(message) => {
            nvm.io.err(&message);
            String::new()
        }
    };
    match index {
        1..=5 => code(nvm, letter(index)),
        _ => code(nvm, letter(2)).replace('0', "1"),
    }
}

/// Text wrapped in a color, when colors are on, as nvm prints it with
/// `printf %b`. `NVM_HAS_COLORS=1` from a caller turns colors on.
///
/// nvm: nvm_wrap_with_color_code
pub fn wrap_with_color_code(nvm: &mut Nvm, letter: &str, text: &str) -> String {
    let code = color_code(letter).ok().flatten();
    let colors_on = nvm.vars.get("NVM_HAS_COLORS") == "1" || has_colors(nvm);
    match code {
        Some(code) if colors_on => printf_b(&format!("\\033[{code}{text}\\033[0m")),
        _ => text.to_string(),
    }
}

/// What `printf %b` makes of a string: backslash escapes are interpreted.
pub fn printf_b(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('a') => out.push('\u{07}'),
            Some('b') => out.push('\u{08}'),
            Some('c') => break,
            Some('f') => out.push('\u{0c}'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('v') => out.push('\u{0b}'),
            Some('0') => {
                let mut value = 0u32;
                for _ in 0..3 {
                    match chars.peek().and_then(|d| d.to_digit(8)) {
                        Some(digit) => {
                            value = value * 8 + digit;
                            chars.next();
                        }
                        None => break,
                    }
                }
                out.push(char::from_u32(value).unwrap_or('\0'));
            }
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printf_b_interprets_octal_escapes() {
        assert_eq!(printf_b("\\033[0;31mx\\033[0m"), "\u{1b}[0;31mx\u{1b}[0m");
        assert_eq!(printf_b("a\\nb"), "a\nb");
    }

    #[test]
    fn codes() {
        assert_eq!(color_code("b"), Ok(Some("0;34m")));
        assert_eq!(color_code("0"), Ok(None));
        assert!(color_code("z").is_err());
    }
}
