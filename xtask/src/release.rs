//! `cargo xtask release`: builds what a release is made of, in `target/dist/`.
//!
//! 1. Builds the nvmimic program for Apple silicon and Intel Macs and joins
//!    the two into one file that runs on both.
//! 2. Signs it with an Apple Developer ID, so macOS knows who made it.
//! 3. Sends it to Apple to be checked (notarized), so macOS opens it without
//!    a warning.
//! 4. Packs it with `LICENSE` and `README.md`, and writes the checksums, a
//!    `VERSION` file and a Homebrew formula.
//!
//! Nothing here uploads anything. The files in `target/dist/` are attached
//! to a release by hand.
//!
//! Signing needs two settings that stay out of the repository:
//! `NVMIMIC_SIGN_IDENTITY`, the name of the Developer ID certificate in the
//! keychain, and `NVMIMIC_NOTARY_PROFILE`, the name a notary login was
//! stored under with `xcrun notarytool store-credentials`. `--unsigned`
//! skips both steps for trying the rest out.

use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const TARGETS: [&str; 2] = ["aarch64-apple-darwin", "x86_64-apple-darwin"];
/// The archive's name carries no version, so the newest one always has the
/// same address under `releases/latest/download/`.
const ARCHIVE: &str = "nvmimic-darwin-universal.tar.gz";

pub fn run(args: &[&str]) -> ExitCode {
    match release(args.contains(&"--unsigned")) {
        Ok(dist) => {
            println!("release: done, see {}", dist.display());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("release: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Runs a program and fails with its name when it does.
fn run_tool(command: &mut Command) -> Result<String, String> {
    let name = command.get_program().to_string_lossy().into_owned();
    let output = command.output().map_err(|e| format!("{name}: {e}"))?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if output.status.success() {
        Ok(text)
    } else {
        Err(format!("{name} failed:\n{text}"))
    }
}

fn setting(name: &str) -> Result<String, String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{name} is not set. See the top of xtask/src/release.rs."))
}

/// The team id at the end of a certificate's name, as in
/// `Developer ID Application: Some Name (ABCDE12345)`.
fn team_id(identity: &str) -> Result<String, String> {
    identity
        .rsplit_once('(')
        .and_then(|(_, rest)| rest.strip_suffix(')'))
        .filter(|id| id.len() == 10 && id.bytes().all(|b| b.is_ascii_alphanumeric()))
        .map(str::to_string)
        .ok_or_else(|| {
            format!("`{identity}` does not end in a team id like (ABCDE12345). Use the certificate's full name.")
        })
}

fn sha256(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn release(unsigned: bool) -> Result<PathBuf, String> {
    let root = crate::repo_root();
    let version = env!("CARGO_PKG_VERSION");
    let identity = match unsigned {
        true => None,
        false => Some((
            setting("NVMIMIC_SIGN_IDENTITY")?,
            setting("NVMIMIC_NOTARY_PROFILE")?,
        )),
    };
    let team = match &identity {
        Some((identity, _)) => Some(team_id(identity)?),
        None => None,
    };

    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    for target in TARGETS {
        println!("release: building for {target}");
        let mut build = Command::new(&cargo);
        build
            .args([
                "build",
                "--release",
                "--package",
                "nvmimic-cli",
                "--target",
                target,
            ])
            .current_dir(&root);
        // The team id is built in, so `nvmimic self update` only accepts a
        // program signed by the same team.
        match &team {
            Some(team) => build.env("NVMIMIC_TEAM_ID", team),
            None => build.env_remove("NVMIMIC_TEAM_ID"),
        };
        run_tool(&mut build)?;
    }

    let dist = root.join("target/dist");
    let _ = fs::remove_dir_all(&dist);
    let staging = dist.join("nvmimic");
    fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    let binary = staging.join("nvmimic");
    let mut lipo = Command::new("lipo");
    lipo.arg("-create").arg("-output").arg(&binary);
    for target in TARGETS {
        lipo.arg(root.join("target").join(target).join("release/nvmimic"));
    }
    run_tool(&mut lipo)?;

    if let Some((identity, profile)) = &identity {
        println!("release: signing as {identity}");
        run_tool(
            Command::new("codesign")
                .args(["--force", "--options", "runtime", "--timestamp", "--sign"])
                .arg(identity)
                .arg(&binary),
        )?;
        run_tool(
            Command::new("codesign")
                .args(["--verify", "--strict", "--verbose=2"])
                .arg(&binary),
        )?;

        // Apple takes a zip. A bare program cannot carry the notary's
        // ticket itself, so macOS looks it up online the first time.
        println!("release: sending to Apple's notary, this takes a few minutes");
        let zip = dist.join("notarize.zip");
        run_tool(
            Command::new("ditto")
                .args(["-c", "-k", "--keepParent"])
                .arg(&binary)
                .arg(&zip),
        )?;
        let answer = run_tool(
            Command::new("xcrun")
                .args(["notarytool", "submit", "--wait", "--keychain-profile"])
                .arg(profile)
                .arg(&zip),
        )?;
        if !answer.contains("status: Accepted") {
            return Err(format!("the notary did not accept the program:\n{answer}"));
        }
        let _ = fs::remove_file(&zip);
    } else {
        println!("release: --unsigned, so not signed and not notarized. Not for publishing.");
    }

    let said = run_tool(Command::new(&binary).arg("--version"))?;
    if said.trim() != version {
        return Err(format!(
            "the built program says version `{}`, expected {version}",
            said.trim()
        ));
    }

    for file in ["LICENSE", "README.md"] {
        fs::copy(root.join(file), staging.join(file)).map_err(|e| format!("{file}: {e}"))?;
    }
    let archive = dist.join(ARCHIVE);
    run_tool(
        Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(&dist)
            .arg("nvmimic")
            // Keep macOS's own extra files out of the archive.
            .env("COPYFILE_DISABLE", "1"),
    )?;
    // nvmimic carries code and text from nvm, so nvm's copyright notice has
    // to travel with every copy of it.
    let listing = run_tool(Command::new("tar").arg("-tzf").arg(&archive))?;
    if !listing.lines().any(|line| line == "nvmimic/LICENSE") {
        return Err(format!("{ARCHIVE} has no LICENSE in it"));
    }
    let license = fs::read_to_string(staging.join("LICENSE")).unwrap_or_default();
    if !license.contains("Tim Caswell") || !license.contains("Jordan Harband") {
        return Err("LICENSE lost nvm's copyright notice".to_string());
    }

    let sum = sha256(&archive)?;
    fs::write(dist.join("SHA256SUMS"), format!("{sum}  {ARCHIVE}\n")).map_err(|e| e.to_string())?;
    fs::write(dist.join("VERSION"), format!("{version}\n")).map_err(|e| e.to_string())?;
    fs::write(dist.join("nvmimic.rb"), formula(version, &sum)).map_err(|e| e.to_string())?;
    fs::remove_dir_all(&staging).map_err(|e| e.to_string())?;

    println!("release: {ARCHIVE} sha256 {sum}");
    println!(
        "release: attach {ARCHIVE}, SHA256SUMS and VERSION to the GitHub release v{version}, \
         and copy nvmimic.rb into the Homebrew tap's Formula folder."
    );
    Ok(dist)
}

/// The Homebrew formula for this release. It installs the program only.
/// Everything that touches `$NVM_DIR` is `nvmimic self install`'s job.
fn formula(version: &str, sha256: &str) -> String {
    let releases = nvmimic::update::RELEASES;
    let homepage = releases.trim_end_matches("/releases");
    format!(
        r##"class Nvmimic < Formula
  desc "Drop-in replacement for nvm that makes new shells start fast"
  homepage "{homepage}"
  url "{releases}/download/v{version}/{ARCHIVE}"
  version "{version}"
  sha256 "{sha256}"
  license "MIT"

  depends_on :macos

  def install
    bin.install "nvmimic"
  end

  def caveats
    <<~EOS
      Run this once to set nvmimic up:
        nvmimic self install
    EOS
  end

  test do
    assert_equal version.to_s, shell_output("#{{bin}}/nvmimic --version").strip
  end
end
"##
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_id_comes_from_the_certificate_name() {
        assert_eq!(
            team_id("Developer ID Application: Some Name (ABCDE12345)").unwrap(),
            "ABCDE12345"
        );
        assert!(team_id("Some Name").is_err());
    }

    #[test]
    fn formula_names_the_release() {
        let text = formula("1.2.3", "abc");
        assert!(text.contains("/download/v1.2.3/nvmimic-darwin-universal.tar.gz"));
        assert!(text.contains("sha256 \"abc\""));
        assert!(text.contains("nvmimic self install"));
    }
}
