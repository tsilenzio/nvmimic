//! Checksums of downloaded archives. nvm computes them with whichever
//! checksum program it finds, and says which one on stderr, so the same
//! programs run here.

use super::paths;
use super::remote;
use super::tools;
use crate::context::Nvm;
use crate::text::install as text;
use std::path::Path;

/// The programs nvm tries, in its order. The first six give SHA-256.
pub const PROGRAMS: [&str; 8] = [
    "sha256sum",
    "shasum",
    "sha256",
    "gsha256sum",
    "openssl",
    "bssl",
    "sha1sum",
    "sha1",
];

/// Whether a program is on `PATH` and its name is not a shell alias.
///
/// nvm: nvm_has_non_aliased
fn has_non_aliased(nvm: &Nvm, name: &str) -> bool {
    paths::on_path(nvm.vars.get("PATH"), name) && !nvm.aliases.iter().any(|alias| alias == name)
}

fn first_program(nvm: &Nvm) -> Option<&'static str> {
    PROGRAMS.into_iter().find(|name| has_non_aliased(nvm, name))
}

/// The checksum program nvm would use, or status 1 with nvm's two error
/// lines when there is none.
///
/// nvm: nvm_get_checksum_binary
pub fn get_checksum_binary(nvm: &mut Nvm) -> Result<&'static str, i32> {
    first_program(nvm).ok_or_else(|| {
        nvm.io.err(text::NO_SHA256_TOOL);
        nvm.io.err(text::NO_SHA1_TOOL);
        1
    })
}

/// `sha-256` or `sha-1`, by the program found.
///
/// nvm: nvm_get_checksum_alg
pub fn get_checksum_alg(nvm: &mut Nvm) -> Result<&'static str, i32> {
    match first_program(nvm) {
        Some("sha1sum" | "sha1") => Ok("sha-1"),
        Some(_) => Ok("sha-256"),
        // nvm asks a second time here, which is what prints the errors.
        None => get_checksum_binary(nvm),
    }
}

/// awk's `{print $1}` or `{print $NF}` over a program's output.
fn field(output: &[u8], last: bool) -> String {
    String::from_utf8_lossy(output)
        .lines()
        .map(|line| {
            let mut fields = line.split([' ', '\t']).filter(|field| !field.is_empty());
            let picked = if last {
                fields.next_back()
            } else {
                fields.next()
            };
            format!("{}\n", picked.unwrap_or(""))
        })
        .collect()
}

/// What nvm prints on stdout for a file's checksum, and its status.
///
/// nvm: nvm_compute_checksum
pub fn compute_checksum(nvm: &mut Nvm, file: &str) -> (String, i32) {
    if file.is_empty() {
        nvm.io.err(text::CHECKSUM_FILE_EMPTY);
        return (String::new(), 2);
    } else if !Path::new(file).is_file() {
        nvm.io.err(text::CHECKSUM_FILE_MISSING);
        return (String::new(), 1);
    }
    let Some(program) = first_program(nvm) else {
        return (String::new(), 0);
    };
    let (shown, args, last): (&str, Vec<&str>, bool) = match program {
        "shasum" => ("shasum -a 256", vec!["-a", "256", file], false),
        "sha256" => ("sha256 -q", vec!["-q", file], false),
        "openssl" => ("openssl dgst -sha256", vec!["dgst", "-sha256", file], true),
        "bssl" => ("bssl sha256sum", vec!["sha256sum", file], false),
        "sha1" => ("sha1 -q", vec!["-q", file], false),
        other => (other, vec![file], false),
    };
    nvm.io.err(&text::computing_checksum(shown));
    let ran = tools::run(nvm, program, &args);
    match (program, ran) {
        // nvm pipes every program but `sha1` through awk, whose status wins.
        ("sha1", Some(ran)) => (
            String::from_utf8_lossy(&ran.stdout).into_owned(),
            ran.status,
        ),
        ("sha1", None) => (String::new(), 127),
        (_, Some(ran)) => (field(&ran.stdout, last), 0),
        (_, None) => (String::new(), 0),
    }
}

/// Checks a file against the checksum the mirror published.
///
/// nvm: nvm_compare_checksum
pub fn compare_checksum(nvm: &mut Nvm, file: &str, checksum: &str) -> i32 {
    if file.is_empty() {
        nvm.io.err(text::CHECKSUM_FILE_EMPTY);
        return 4;
    } else if !Path::new(file).is_file() {
        nvm.io.err(text::CHECKSUM_FILE_MISSING);
        return 3;
    }
    let (computed, _) = compute_checksum(nvm, file);
    let computed = computed.trim_end_matches('\n');
    if checksum.is_empty() {
        nvm.io.err(text::CHECKSUM_EMPTY);
        return 2;
    }
    if computed.is_empty() {
        // Seen on Raspberry Pi builds whose checksum program prints nothing.
        nvm.io.err(&text::computed_checksum_empty(file));
        nvm.io.err(text::CHECKSUM_UNVERIFIED);
        return 0;
    } else if computed != checksum && computed != format!("\\{checksum}") {
        nvm.io.err(&text::checksums_differ(computed, checksum));
        return 1;
    }
    nvm.io.err(text::CHECKSUMS_MATCHED);
    0
}

/// The checksum the mirror lists for one archive, empty when it lists none.
///
/// nvm: nvm_get_checksum
pub fn get_checksum(
    nvm: &mut Nvm,
    flavor: &str,
    kind: &str,
    version: &str,
    slug: &str,
    compression: &str,
) -> (String, i32) {
    if !matches!(flavor, "node" | "iojs") {
        nvm.io.err(text::SUPPORTED_FLAVORS);
        return (String::new(), 2);
    }
    let Ok(mirror) = remote::get_mirror(nvm, flavor, kind) else {
        return (String::new(), 1);
    };
    let sums = match get_checksum_alg(nvm) {
        Ok("sha-256") => "SHASUMS256.txt",
        _ => "SHASUMS.txt",
    };
    let url = format!("{mirror}/{version}/{sums}");
    let listing = remote::download_capture(nvm, &["-L", "-s", &url, "-o", "-"]);
    let tarball = format!("{slug}.{compression}");
    let found = String::from_utf8_lossy(&listing)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split([' ', '\t']).filter(|field| !field.is_empty());
            let sum = fields.next()?;
            (fields.next() == Some(tarball.as_str())).then(|| format!("{sum}\n"))
        })
        .collect();
    (found, 0)
}
