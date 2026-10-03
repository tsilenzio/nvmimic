//! Just enough of npm's semver ranges to read a package's `engines.node`,
//! such as `^20.17.0 || >=22.9.0`. Anything this does not understand counts
//! as not satisfied, which sends the caller down its slower, exact path.

type Triple = (u64, u64, u64);

/// A plain `X.Y.Z` version, with an optional leading `v`. A prerelease or
/// build suffix gives `None`: npm's ranges leave prereleases out.
pub fn parse_version(text: &str) -> Option<Triple> {
    let text = text.strip_prefix('v').unwrap_or(text);
    let mut parts = text.split('.');
    let triple = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    parts.next().is_none().then_some(triple)
}

/// A version in a range, where missing or `x`/`*` parts are wildcards.
fn parse_partial(text: &str) -> Option<[Option<u64>; 3]> {
    let text = text.strip_prefix('v').unwrap_or(text);
    let mut out = [None; 3];
    let mut wild = false;
    for (index, part) in text.split('.').enumerate() {
        if index > 2 {
            return None;
        }
        if matches!(part, "x" | "X" | "*" | "") {
            wild = true;
        } else if wild {
            return None;
        } else {
            out[index] = Some(part.parse().ok()?);
        }
    }
    Some(out)
}

fn floor(partial: [Option<u64>; 3]) -> Triple {
    (
        partial[0].unwrap_or(0),
        partial[1].unwrap_or(0),
        partial[2].unwrap_or(0),
    )
}

/// The first version past everything a partial version covers.
fn ceiling(partial: [Option<u64>; 3]) -> Option<Triple> {
    match partial {
        [None, _, _] => None,
        [Some(major), None, _] => Some((major + 1, 0, 0)),
        [Some(major), Some(minor), None] => Some((major, minor + 1, 0)),
        [Some(major), Some(minor), Some(patch)] => Some((major, minor, patch + 1)),
    }
}

/// One comparator such as `>=22.9.0`, `^20.17.0`, `~1.2` or `18.x`.
fn comparator(version: Triple, token: &str) -> Option<bool> {
    let (op, rest) = ["<=", ">=", "<", ">", "=", "^", "~"]
        .iter()
        .find_map(|op| token.strip_prefix(op).map(|rest| (*op, rest)))
        .unwrap_or(("", token));
    let partial = parse_partial(rest)?;
    let low = floor(partial);
    Some(match op {
        ">=" => version >= low,
        ">" => match ceiling(partial) {
            Some(high) if partial[2].is_none() => version >= high,
            _ => version > low,
        },
        "<" => version < low,
        "<=" => match ceiling(partial) {
            Some(high) if partial[2].is_none() => version < high,
            _ => version <= low,
        },
        "^" => {
            let high = match partial {
                [Some(0), Some(0), Some(patch)] => (0, 0, patch + 1),
                [Some(0), Some(minor), _] => (0, minor + 1, 0),
                [Some(major), _, _] => (major + 1, 0, 0),
                [None, _, _] => return Some(true),
            };
            version >= low && version < high
        }
        "~" => {
            let high = match partial {
                [Some(major), Some(minor), _] => (major, minor + 1, 0),
                [Some(major), None, _] => (major + 1, 0, 0),
                [None, _, _] => return Some(true),
            };
            version >= low && version < high
        }
        _ => match ceiling(partial) {
            Some(high) => version >= low && version < high,
            None => true,
        },
    })
}

/// Whether `version` satisfies an npm range: alternatives joined by `||`,
/// each a list of comparators that must all hold.
pub fn satisfies(version: &str, range: &str) -> bool {
    let Some(version) = parse_version(version) else {
        return false;
    };
    range.split("||").any(|set| {
        let tokens: Vec<&str> = set.split_whitespace().collect();
        if tokens.is_empty() || tokens.contains(&"-") {
            return tokens.is_empty();
        }
        tokens
            .iter()
            .all(|token| comparator(version, token) == Some(true))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npm_engine_ranges() {
        let npm11 = "^20.17.0 || >=22.9.0";
        assert!(satisfies("v22.23.3", npm11));
        assert!(satisfies("20.17.0", npm11));
        assert!(!satisfies("v20.16.0", npm11));
        assert!(!satisfies("v21.5.0", npm11));
        assert!(!satisfies("v22.8.0", npm11));
        let npm10 = "^18.17.0 || >=20.5.0";
        assert!(satisfies("v18.20.8", npm10));
        assert!(!satisfies("v19.0.0", npm10));
        assert!(satisfies("v14.21.3", ">=10"));
        assert!(satisfies("v14.21.3", "14.x"));
        assert!(!satisfies("v14.21.3", ">=99.0.0"));
        assert!(satisfies("v16.2.0", ">=14 <17"));
    }

    #[test]
    fn unknown_forms_do_not_satisfy() {
        assert!(!satisfies("v26.0.0-pre", ">=20"));
        assert!(!satisfies("v20.0.0", "18.0.0 - 22.0.0"));
        assert!(!satisfies("v20.0.0", "latest"));
        assert!(!satisfies("none", ">=1"));
    }
}
