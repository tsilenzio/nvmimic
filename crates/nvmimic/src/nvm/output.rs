//! The version table `nvm ls` and `nvm ls-remote` print.

use super::{colors, local};
use crate::context::Nvm;

/// awk's `printf "%Ns"`: right-aligned to a width counted in bytes.
fn pad(width: usize, text: &str) -> String {
    if text.len() >= width {
        text.to_string()
    } else {
        format!("{}{text}", " ".repeat(width - text.len()))
    }
}

/// Fills one awk format holding `%15s`, `%13s` and `%s` fields.
fn fill(format: &str, args: &[&str]) -> String {
    let mut out = String::new();
    let mut args = args.iter();
    let mut rest = format;
    while let Some(at) = rest.find('%') {
        out.push_str(&rest[..at]);
        let spec = &rest[at + 1..];
        let digits: String = spec.chars().take_while(|c| c.is_ascii_digit()).collect();
        let after = &spec[digits.len()..];
        if let Some(tail) = after.strip_prefix('s') {
            let arg = args.next().copied().unwrap_or("");
            match digits.parse::<usize>() {
                Ok(width) => out.push_str(&pad(width, arg)),
                Err(_) => out.push_str(arg),
            }
            rest = tail;
        } else {
            out.push('%');
            rest = spec;
        }
    }
    out.push_str(rest);
    out
}

/// Prints one row per line of `listing`: the installed, current and system
/// markers, and LTS names.
///
/// nvm: nvm_print_versions
pub fn print_versions(nvm: &mut Nvm, listing: &str) {
    let current = local::ls_current(nvm);
    let installed_color = colors::get_color(nvm, 1);
    let system_color = colors::get_color(nvm, 2);
    let current_color = colors::get_color(nvm, 3);
    let _not_installed_color = colors::get_color(nvm, 4);
    let default_color = colors::get_color(nvm, 5);
    let _lts_color = colors::get_color(nvm, 6);
    let has_colors = colors::has_colors(nvm);

    let installed_listing = local::ls(nvm, "").0;
    let installed: Vec<&str> = split_rows(&installed_listing);

    let esc = "\u{1b}[";
    let colored = |color: &str, inner: &str, plain: &str| -> String {
        if !color.is_empty() {
            format!("{esc}{color}{inner}{esc}0m")
        } else {
            plain.to_string()
        }
    };
    let (fmt_installed, fmt_system, fmt_current) = if has_colors {
        (
            colored(&installed_color, "%15s", "%15s"),
            colored(&system_color, "%15s", "%15s"),
            colored(&current_color, "->%13s", "%15s"),
        )
    } else {
        (
            "%15s *".to_string(),
            "%15s *".to_string(),
            "->%13s *".to_string(),
        )
    };
    let latest_lts_color = current_color.replacen("0;", "1;", 1);
    let fmt_latest_lts = if has_colors && !latest_lts_color.is_empty() {
        format!("{esc}{latest_lts_color} (Latest LTS: %s){esc}0m")
    } else {
        " (Latest LTS: %s)".to_string()
    };
    let fmt_old_lts = if has_colors && !default_color.is_empty() {
        format!("{esc}{default_color} (LTS: %s){esc}0m")
    } else {
        " (LTS: %s)".to_string()
    };
    let fmt_system_target = if has_colors && !system_color.is_empty() {
        format!(" ({esc}{system_color}-> %s{esc}0m)")
    } else {
        " (-> %s)".to_string()
    };

    // awk keeps `formatted` between rows, so a row it cannot place repeats
    // the row before it.
    let mut formatted = String::new();
    for row in split_rows(listing) {
        let fields = split_blank(row);
        let cols = fields.len();
        let version = fields.first().copied().unwrap_or("");
        let is_installed = installed.contains(&version);

        let fmt_version = if version == current {
            fmt_current.as_str()
        } else if version == "system" {
            fmt_system.as_str()
        } else if is_installed {
            fmt_installed.as_str()
        } else {
            "%15s"
        };
        let padding = if !has_colors && is_installed {
            ""
        } else {
            "  "
        };

        if cols == 1 {
            formatted = fill(fmt_version, &[version]);
        } else if version == "system" && cols >= 2 {
            formatted = fill(
                &format!("{fmt_version}{fmt_system_target}"),
                &[version, fields[1]],
            );
        } else if cols == 2 {
            formatted = fill(
                &format!("{fmt_version}{padding}{fmt_old_lts}"),
                &[version, fields[1]],
            );
        } else if cols == 3 && fields[2] == "*" {
            formatted = fill(
                &format!("{fmt_version}{padding}{fmt_latest_lts}"),
                &[version, fields[1]],
            );
        }
        nvm.io.echo(&formatted);
    }
}

/// `split(text, rows, "|")` after nvm turned newlines into `|`: an empty
/// listing has no rows.
fn split_rows(text: &str) -> Vec<&str> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.split(['\n', '|']).collect()
    }
}

/// `split(row, fields, "[[:blank:]]+")`, where a leading blank makes an
/// empty first field.
fn split_blank(row: &str) -> Vec<&str> {
    if row.is_empty() {
        return Vec::new();
    }
    let mut fields = Vec::new();
    let mut rest = row;
    loop {
        match rest.find([' ', '\t']) {
            Some(at) => {
                fields.push(&rest[..at]);
                rest = rest[at..].trim_start_matches([' ', '\t']);
                if rest.is_empty() {
                    fields.push("");
                    break;
                }
            }
            None => {
                fields.push(rest);
                break;
            }
        }
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_and_fields() {
        assert_eq!(fill("->%13s *", &["v22.1.0"]), "->      v22.1.0 *");
        assert_eq!(fill("%15s", &["system"]), "         system");
        assert_eq!(split_blank("v1 x"), vec!["v1", "x"]);
        assert_eq!(split_blank(" v1"), vec!["", "v1"]);
        assert_eq!(split_blank("v1 "), vec!["v1", ""]);
    }
}
