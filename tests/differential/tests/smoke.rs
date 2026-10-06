//! A quick end-to-end check with real Node, run through `cargo xtask smoke`.
//!
//! The parity cases install stand-in Node releases whose `node` is a shell
//! script. This test installs nvmimic with `nvmimic self install` into a
//! throwaway home and then installs three real Node releases, switches
//! between them and asks `node` and `npm` for their versions. It compares
//! nothing with real nvm. It answers one question: does an installed nvmimic
//! still do its everyday job.
//!
//! The Node archives come through the mirror's network mode, so each one is
//! downloaded from nodejs.org once and kept in `target/parity-network/`.

use nvmimic_parity::{guard, harness, mirror};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const MAJORS: [&str; 3] = ["20", "22", "24"];

/// Prints `ok <label>` or `FAIL <label>` for one comparison.
const EXPECT: &str = r#"expect() {
  if [ "$2" = "$3" ]; then
    echo "ok $1"
  else
    echo "FAIL $1: expected '$2', got '$3'"
  fi
}
export NVM_DIR="$HOME/.nvm"
. "$NVM_DIR/nvm.sh" || echo "FAIL sourcing nvm.sh returned $?"
"#;

const ZSH_BODY: &str = r#"
for major in 20 22 24; do
  nvm install "$major" || echo "FAIL nvm install $major returned $?"
done
for major in 20 22 24; do
  eval "node=\$NODE_$major npm=\$NPM_$major"
  nvm use "$major" >/dev/null || echo "FAIL nvm use $major returned $?"
  expect "use $major: node --version" "$node" "$(node --version)"
  expect "use $major: npm --version" "$npm" "$(npm --version)"
  expect "use $major: nvm current" "$node" "$(nvm current)"
done
case "$(nvm ls --no-colors)" in
  *"$NODE_20"*"$NODE_22"*"$NODE_24"*) echo "ok nvm ls" ;;
  *) echo "FAIL nvm ls: $(nvm ls --no-colors)" ;;
esac
expect "nvm which 22" "$NVM_DIR/versions/node/$NODE_22/bin/node" "$(nvm which 22)"
expect "nvm exec 20" "$NODE_20" "$(nvm exec --silent 20 node --version)"
expect "nvm current after exec" "$NODE_24" "$(nvm current)"
nvm alias default 22 >/dev/null || echo "FAIL nvm alias default 22 returned $?"
expect "alias file" "22" "$(cat "$NVM_DIR/alias/default")"
expect "nvm version default" "$NODE_22" "$(nvm version default)"
"#;

/// What a new shell sees once `default` points at 22.
const NEW_SHELL_BODY: &str = r#"
expect "new shell: node --version" "$NODE_22" "$(node --version)"
expect "new shell: npm --version" "$NPM_22" "$(npm --version)"
"#;

fn run(home: &Path, vars: &BTreeMap<String, String>, program: &Path, args: &[&str]) -> Output {
    Command::new(program)
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("TERM", "dumb")
        .envs(vars)
        .current_dir(home)
        .output()
        .unwrap()
}

/// Runs a script and checks that it printed `checks` lines starting with
/// `ok` and none starting with `FAIL`.
fn check(home: &Path, vars: &BTreeMap<String, String>, shell: &str, body: &str, checks: usize) {
    let shell = harness::shell(shell).unwrap_or_else(|| panic!("no {shell} on this machine"));
    let script = format!("{EXPECT}{body}");
    let output = run(home, vars, &shell.path, &["-c", &script]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let ok = stdout
        .lines()
        .filter(|line| line.starts_with("ok "))
        .count();
    let failed = stdout.lines().any(|line| line.starts_with("FAIL"));
    assert!(
        ok == checks && !failed && output.status.success(),
        "{}: {ok} of {checks} checks passed, {}\n--- stdout\n{stdout}\n--- stderr\n{}",
        shell.name,
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    eprintln!("{}: {ok} of {checks} checks passed", shell.name);
}

#[test]
#[ignore = "installs real Node releases, run through `cargo xtask smoke`"]
fn real_node_smoke() {
    assert!(
        mirror::network_pass(),
        "run this through `cargo xtask smoke`, which lets the mirror fetch from nodejs.org"
    );
    let mirror = mirror::shared();
    let home = std::env::temp_dir().join(format!("nvmimic-diff-smoke-{}", std::process::id()));
    fs::create_dir_all(&home).unwrap();
    let guard = guard::Guard::start(&home);

    // The newest release of each major and the npm it ships, from the index.
    let index = run(
        &home,
        &BTreeMap::new(),
        Path::new("/usr/bin/curl"),
        &[
            "--fail",
            "--silent",
            &format!("{}{}/index.tab", mirror.url, mirror::REAL),
        ],
    );
    let index = String::from_utf8_lossy(&index.stdout).into_owned();
    let mut vars = BTreeMap::from([(
        "NVM_NODEJS_ORG_MIRROR".to_string(),
        format!("{}{}", mirror.url, mirror::REAL),
    )]);
    for major in MAJORS {
        let row = index
            .lines()
            .find(|line| line.starts_with(&format!("v{major}.")))
            .unwrap_or_else(|| panic!("the index lists no Node {major}"));
        let columns: Vec<&str> = row.split('\t').collect();
        vars.insert(format!("NODE_{major}"), columns[0].to_string());
        vars.insert(format!("NPM_{major}"), columns[3].to_string());
    }

    let installed = run(
        &home,
        &vars,
        &harness::nvmimic_bin(),
        &["self", "install", "--force"],
    );
    assert!(
        installed.status.success(),
        "self install failed\n{}{}",
        String::from_utf8_lossy(&installed.stdout),
        String::from_utf8_lossy(&installed.stderr)
    );

    check(&home, &vars, "zsh", ZSH_BODY, 15);
    check(&home, &vars, "zsh", NEW_SHELL_BODY, 2);
    for (shell, major) in [("bash", "20"), ("dash", "24"), ("sh", "22")] {
        let body = format!(
            "nvm use {major} >/dev/null || echo \"FAIL nvm use {major} returned $?\"\n\
             expect 'node --version' \"$NODE_{major}\" \"$(node --version)\"\n"
        );
        check(&home, &vars, shell, &body, 1);
    }

    let uninstalled = run(
        &home,
        &vars,
        &harness::nvmimic_bin(),
        &["self", "uninstall", "--force"],
    );
    assert!(
        uninstalled.status.success(),
        "self uninstall failed\n{}{}",
        String::from_utf8_lossy(&uninstalled.stdout),
        String::from_utf8_lossy(&uninstalled.stderr)
    );
    assert!(
        home.join(".nvm/versions/node").is_dir(),
        "self uninstall removed the installed Node versions"
    );
    eprintln!("{}", guard.finish());
    assert!(
        guard.stopped().is_none(),
        "the gatekeeper guard stopped the run"
    );
}
