//! Reading nvm.sh's structure: its top-level functions and the arms of the
//! `nvm()` dispatcher, with the lines each one spans.

/// One traceable piece of nvm.sh. Functions are named as in nvm.sh, the
/// dispatcher's arms as `nvm:<command>`, and its help scan as `nvm:help`.
#[derive(Clone, Debug)]
pub struct Unit {
    pub name: String,
    /// First and last line, counted from 1.
    pub start: usize,
    pub end: usize,
}

pub fn units(source: &str) -> Vec<Unit> {
    let lines: Vec<&str> = source.split('\n').collect();
    let mut units = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if let Some(name) = lines[i].strip_suffix("() {")
            && !name.is_empty()
            && name.chars().all(|c| c.is_ascii_lowercase() || c == '_')
        {
            let Some(end) = (i + 1..lines.len()).find(|&j| lines[j] == "}") else {
                break;
            };
            units.push(Unit {
                name: name.to_string(),
                start: i + 1,
                end: end + 1,
            });
            if name == "nvm" {
                units.extend(dispatcher_arms(&lines, i, end));
            }
            i = end;
        }
        i += 1;
    }
    units
}

/// The case arms of `nvm()`, at four spaces of indentation, plus the help
/// scan before them.
fn dispatcher_arms(lines: &[&str], start: usize, end: usize) -> Vec<Unit> {
    let mut arms: Vec<(String, usize)> = Vec::new();
    for (j, line) in lines.iter().enumerate().take(end).skip(start) {
        if line.starts_with("    ") && !line.starts_with("     ") && line.ends_with(')') {
            let label = line.trim().trim_end_matches(')');
            let first = label
                .split('|')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"');
            if label == "*" || label.starts_with('"') {
                let name = if first == "*" { "*" } else { first };
                arms.push((format!("nvm:{name}"), j));
            }
        }
    }
    let mut units = Vec::new();
    if let Some((_, first_arm)) = arms.first() {
        let help = (start..*first_arm).find(|&j| lines[j].contains("'-h'|'help'|'--help')"));
        if let Some(help) = help {
            let close = (help..*first_arm)
                .find(|&j| lines[j].trim() == ";;")
                .unwrap_or(help);
            units.push(Unit {
                name: "nvm:help".to_string(),
                start: help + 1,
                end: close + 1,
            });
        }
    }
    for (index, (name, line)) in arms.iter().enumerate() {
        let next = arms.get(index + 1).map_or(end, |(_, next)| *next);
        units.push(Unit {
            name: name.clone(),
            start: line + 1,
            end: next,
        });
    }
    units
}

/// The unit a line falls in, preferring the innermost (an arm over `nvm`).
pub fn unit_at(units: &[Unit], line: usize) -> Option<&Unit> {
    units
        .iter()
        .filter(|unit| unit.start <= line && line <= unit.end)
        .min_by_key(|unit| unit.end - unit.start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_functions_and_arms() {
        let source = "a_b() {\n  x\n}\nnvm() {\n  case $1 in\n    \"use\")\n      y\n    ;;\n    *)\n    ;;\n  esac\n}\n";
        let units = units(source);
        let names: Vec<&str> = units.iter().map(|u| u.name.as_str()).collect();
        assert_eq!(names, ["a_b", "nvm", "nvm:use", "nvm:*"]);
        assert_eq!(unit_at(&units, 7).unwrap().name, "nvm:use");
    }
}
