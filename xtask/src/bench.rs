//! `cargo xtask bench`: how long sourcing `nvm.sh` takes, for real nvm and
//! for nvmimic, with real Node installs, under every shell and scenario.
//!
//! The fixture lives in `target/bench/`. Building it the first time runs the
//! vendored nvm to install real Node versions from nodejs.org.

use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const VERSIONS: &[&str] = &["22", "20"];
const DEFAULT: &str = "22";
const BASE_PATH: &str = "/usr/bin:/bin:/usr/sbin:/sbin";

/// Programs a traced command counts as a launch of something external.
const EXTERNAL: &[&str] = &[
    "awk", "basename", "cat", "curl", "cut", "dirname", "find", "getconf", "grep", "head", "id",
    "ls", "manpath", "mkdir", "node", "npm", "od", "sed", "sort", "sw_vers", "tail", "tput", "tr",
    "uname", "uniq", "wget", "which", "xargs",
];

struct ShellRun {
    name: &'static str,
    path: &'static str,
    flags: &'static [&'static str],
    interactive: bool,
}

const SHELLS: &[ShellRun] = &[
    ShellRun {
        name: "bash",
        path: "/bin/bash",
        flags: &[],
        interactive: false,
    },
    ShellRun {
        name: "zsh -f",
        path: "/bin/zsh",
        flags: &["-f"],
        interactive: false,
    },
    ShellRun {
        name: "zsh -i",
        path: "/bin/zsh",
        flags: &["-i"],
        interactive: true,
    },
    ShellRun {
        name: "sh",
        path: "/bin/sh",
        flags: &[],
        interactive: false,
    },
    ShellRun {
        name: "dash",
        path: "/bin/dash",
        flags: &[],
        interactive: false,
    },
];

#[derive(Clone, Copy, PartialEq)]
enum Scenario {
    /// A new terminal: no Node on PATH.
    Fresh,
    /// A shell started from one where nvm already put a Node on PATH.
    Nested,
    /// A Node outside nvm on PATH, the way Homebrew installs one.
    System,
}

impl Scenario {
    const ALL: [Scenario; 3] = [Scenario::Fresh, Scenario::Nested, Scenario::System];

    fn name(self) -> &'static str {
        match self {
            Scenario::Fresh => "fresh",
            Scenario::Nested => "nested",
            Scenario::System => "system",
        }
    }
}

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn home(&self, which: &str) -> PathBuf {
        self.root.join(which).join("home")
    }

    fn zdotdir(&self) -> PathBuf {
        self.root.join("zdotdir")
    }

    fn shims(&self) -> PathBuf {
        self.root.join("shims")
    }

    fn default_bin(&self, which: &str) -> PathBuf {
        let versions = self.home(which).join(".nvm/versions/node");
        let mut found: Vec<PathBuf> = fs::read_dir(&versions)
            .unwrap()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with(&format!("v{DEFAULT}.")))
            })
            .collect();
        found.sort();
        found.pop().unwrap().join("bin")
    }

    fn path(&self, which: &str, scenario: Scenario) -> String {
        match scenario {
            Scenario::Fresh => BASE_PATH.to_string(),
            Scenario::Nested => format!("{}:{BASE_PATH}", self.default_bin(which).display()),
            Scenario::System => format!("{}:{BASE_PATH}", self.root.join("system/bin").display()),
        }
    }

    /// The command hyperfine times, starting from an empty environment.
    fn command(
        &self,
        which: &str,
        shell: &ShellRun,
        scenario: Scenario,
        snippet: &str,
    ) -> Vec<String> {
        let home = self.home(which);
        let mut command = vec![
            "/usr/bin/env".to_string(),
            "-i".to_string(),
            format!("HOME={}", home.display()),
            format!("NVM_DIR={}", home.join(".nvm").display()),
            format!("PATH={}", self.path(which, scenario)),
            "TERM=xterm".to_string(),
            // A login session exports SHELL. Without it bash sets the
            // variable for itself and doesn't export it, and nvmimic then
            // can't tell what the shell holds without a second start.
            format!("SHELL={}", shell.path),
        ];
        if shell.interactive {
            command.push(format!("ZDOTDIR={}", self.zdotdir().display()));
        }
        command.push(shell.path.to_string());
        command.extend(shell.flags.iter().map(|flag| flag.to_string()));
        command.push("-c".to_string());
        command.push(snippet.to_string());
        command
    }
}

pub fn run(args: &[&str]) -> ExitCode {
    let check = args.contains(&"--check");
    let runs = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--runs="))
        .unwrap_or("20");
    let root = crate::repo_root().join("target/bench");

    if !crate::cargo(&["build", "--quiet", "--release", "--package", "nvmimic-cli"]) {
        return ExitCode::FAILURE;
    }
    let fixture = match prepare(&root) {
        Ok(fixture) => fixture,
        Err(message) => {
            eprintln!("bench: {message}");
            return ExitCode::FAILURE;
        }
    };
    let nvmimic_ready = fixture.home("nvmimic").join(".nvm/nvm.sh").is_file();

    let mut rows = Vec::new();
    for shell in SHELLS
        .iter()
        .filter(|shell| Path::new(shell.path).is_file())
    {
        for scenario in Scenario::ALL {
            let mut commands = vec![("empty", fixture.command("nvm", shell, scenario, "true"))];
            let source = ". \"$NVM_DIR/nvm.sh\"";
            commands.push(("nvm", fixture.command("nvm", shell, scenario, source)));
            if nvmimic_ready {
                commands.push((
                    "nvmimic",
                    fixture.command("nvmimic", shell, scenario, source),
                ));
            }
            let means = hyperfine(&root, runs, &commands);
            let launches = |which: &str| launches(&fixture, which, shell, scenario);
            rows.push(Row {
                shell: shell.name,
                scenario: scenario.name(),
                empty: means["empty"],
                nvm: means["nvm"],
                nvmimic: means.get("nvmimic").copied(),
                nvm_launches: launches("nvm"),
                nvmimic_launches: nvmimic_ready.then(|| launches("nvmimic")),
            });
        }
    }
    print_table(&rows);

    if check {
        if !nvmimic_ready {
            eprintln!("bench: --check needs shell/nvm.sh, which does not exist yet");
            return ExitCode::FAILURE;
        }
        return check_budget(&rows);
    }
    ExitCode::SUCCESS
}

struct Row {
    shell: &'static str,
    scenario: &'static str,
    empty: f64,
    nvm: f64,
    nvmimic: Option<f64>,
    nvm_launches: BTreeMap<String, usize>,
    nvmimic_launches: Option<BTreeMap<String, usize>>,
}

fn prepare(root: &Path) -> Result<Fixture, String> {
    let fixture = Fixture {
        root: root.to_path_buf(),
    };
    let base = root.join("base/home");
    let reference = crate::reference_dir();

    if !base.join(".nvm/alias/default").is_file() {
        eprintln!(
            "bench: installing Node {} with real nvm (needs network)",
            VERSIONS.join(", ")
        );
        let _ = fs::remove_dir_all(root.join("base"));
        fs::create_dir_all(base.join(".nvm")).map_err(|e| e.to_string())?;
        for file in ["nvm.sh", "nvm-exec", "bash_completion"] {
            fs::copy(reference.join(file), base.join(".nvm").join(file))
                .map_err(|e| e.to_string())?;
        }
        // An existing nvm download cache saves fetching the archives again.
        // nvm still checks each one against the mirror's checksums.
        if let Some(cache) = std::env::var_os("NVMIMIC_BENCH_CACHE") {
            copy_tree(Path::new(&cache), &base.join(".nvm/.cache"))?;
        }
        let script = format!(
            ". \"$NVM_DIR/nvm.sh\"\n{}\nnvm alias default {DEFAULT}\n",
            VERSIONS
                .iter()
                .map(|v| format!("nvm install {v} || exit 1"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let status = Command::new("/usr/bin/env")
            .args(["-i", &format!("HOME={}", base.display())])
            .arg(format!("NVM_DIR={}", base.join(".nvm").display()))
            .arg(format!("PATH={BASE_PATH}"))
            .args(["/bin/bash", "-c", &script])
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err("installing Node with real nvm failed".to_string());
        }
    }

    for which in ["nvm", "nvmimic"] {
        let home = fixture.home(which);
        if !home.join(".nvm/versions").is_dir() {
            let _ = fs::remove_dir_all(root.join(which));
            fs::create_dir_all(root.join(which)).map_err(|e| e.to_string())?;
            copy_tree(&base, &home)?;
        }
        let nvm_dir = home.join(".nvm");
        for file in ["nvm.sh", "nvm-exec", "bash_completion"] {
            let _ = fs::remove_file(nvm_dir.join(file));
        }
        // nvmimic's own files sit beside the home folder, as in a parity
        // sandbox.
        let own = root.join(which).join("own");
        let _ = fs::remove_dir_all(&own);
        if which == "nvm" {
            for file in ["nvm.sh", "nvm-exec", "bash_completion"] {
                fs::copy(reference.join(file), nvm_dir.join(file)).map_err(|e| e.to_string())?;
            }
        } else {
            let shell_dir = crate::repo_root().join("shell");
            for file in ["nvm-exec", "bash_completion"] {
                let source = shell_dir.join(file);
                if source.is_file() {
                    fs::copy(&source, nvm_dir.join(file)).map_err(|e| e.to_string())?;
                }
            }
            let bin = own.join("bin");
            fs::create_dir_all(&bin).map_err(|e| e.to_string())?;
            fs::create_dir_all(own.join("cache")).map_err(|e| e.to_string())?;
            // The cache folder sits beside the home folder here, so the
            // program is told where it is.
            let quote = nvmimic::shell::quote;
            let program = quote(&bin.join("nvmimic").display().to_string());
            let init = format!("__NVMIMIC_PROGRAM={program} command {program} init");
            let told = format!(
                "NVMIMIC_CACHE_DIR={} {init}",
                quote(&own.join("cache").display().to_string())
            );
            let script = nvmimic::install::nvm_sh(&bin.join("nvmimic"));
            if !script.contains(&init) {
                return Err("nvm.sh no longer runs `init` this way".to_string());
            }
            fs::write(nvm_dir.join("nvm.sh"), script.replace(&init, &told))
                .map_err(|e| e.to_string())?;
            let binary = crate::repo_root().join("target/release/nvmimic");
            fs::copy(&binary, bin.join("nvmimic")).map_err(|e| e.to_string())?;
        }
    }

    // A Node outside nvm's tree stands in for a Homebrew install.
    let system = root.join("system/bin");
    if !system.join("node").exists() {
        fs::create_dir_all(&system).map_err(|e| e.to_string())?;
        let other = base.join(".nvm/versions/node");
        let node = fs::read_dir(&other)
            .map_err(|e| e.to_string())?
            .flatten()
            .map(|entry| entry.path().join("bin/node"))
            .find(|path| !path.starts_with(fixture.default_bin("nvm")) && path.is_file())
            .ok_or("no second Node version to use as the system Node")?;
        std::os::unix::fs::symlink(node, system.join("node")).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(fixture.zdotdir()).map_err(|e| e.to_string())?;
    write_shims(&fixture.shims())?;
    Ok(fixture)
}

/// A logging stand-in for each counted tool, which records its name and then
/// runs the real program.
fn write_shims(dir: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    for tool in EXTERNAL
        .iter()
        .filter(|tool| !matches!(**tool, "node" | "npm"))
    {
        let Some(real) = BASE_PATH
            .split(':')
            .map(|dir| Path::new(dir).join(tool))
            .find(|path| path.is_file())
        else {
            continue;
        };
        let shim = dir.join(tool);
        fs::write(
            &shim,
            format!(
                "#!/bin/sh\nprintf '%s\\n' {tool} >> \"$NVMIMIC_LAUNCH_LOG\"\nexec '{}' \"$@\"\n",
                real.display()
            ),
        )
        .map_err(|e| e.to_string())?;
        fs::set_permissions(&shim, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Copies a directory tree, as a copy-on-write clone where the filesystem
/// supports it so the Node installs cost no extra space.
fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    let clone_flag = if cfg!(target_os = "macos") {
        "-cR"
    } else {
        "-R"
    };
    let status = Command::new("cp")
        .arg(clone_flag)
        .arg(from)
        .arg(to)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cp {} failed", from.display()))
    }
}

/// Times each command with hyperfine and returns the mean in milliseconds.
fn hyperfine(root: &Path, runs: &str, commands: &[(&str, Vec<String>)]) -> BTreeMap<String, f64> {
    let export = root.join("last.json");
    let mut command = Command::new("hyperfine");
    command.args([
        "-N",
        "--warmup",
        "3",
        "--runs",
        runs,
        "--style",
        "none",
        "--export-json",
    ]);
    command.arg(&export);
    for (name, argv) in commands {
        command.args(["--command-name", name]);
        command.arg(
            argv.iter()
                .map(|arg| shell_quote(arg))
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    let status = command.status().expect("hyperfine is not installed");
    assert!(status.success(), "hyperfine failed");

    let json: Value = serde_json::from_str(&fs::read_to_string(&export).unwrap()).unwrap();
    json["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|result| {
            (
                result["command"].as_str().unwrap().to_string(),
                result["mean"].as_f64().unwrap() * 1000.0,
            )
        })
        .collect()
}

/// Counts the programs one source starts. Tools are counted by logging shims
/// put first on `PATH`, which also catches calls whose stderr nvm discards.
/// `node`, `npm` and nvmimic itself are counted from the shell's xtrace,
/// since a shim named `node` would change which node nvm finds. That count
/// misses calls nvm makes with stderr sent to /dev/null, so it is a floor.
fn launches(
    fixture: &Fixture,
    which: &str,
    shell: &ShellRun,
    scenario: Scenario,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();

    let log = fixture.root.join("launches.log");
    let _ = fs::remove_file(&log);
    let mut argv = fixture.command(which, shell, scenario, ". \"$NVM_DIR/nvm.sh\"");
    for arg in argv.iter_mut() {
        if let Some(path) = arg.strip_prefix("PATH=") {
            *arg = format!("PATH={}:{path}", fixture.shims().display());
        }
    }
    argv.insert(2, format!("NVMIMIC_LAUNCH_LOG={}", log.display()));
    // nvmimic's own binary is counted by a logging wrapper that takes its
    // place for this one run.
    let binary = fixture.root.join(which).join("own/bin/nvmimic");
    let wrapped = which == "nvmimic" && wrap_binary(&binary).is_ok();
    let _ = Command::new(&argv[0]).args(&argv[1..]).output();
    if wrapped {
        let _ = fs::rename(binary.with_file_name("nvmimic.real"), &binary);
    }
    for tool in fs::read_to_string(&log).unwrap_or_default().lines() {
        *counts.entry(tool.to_string()).or_insert(0) += 1;
    }
    if which == "nvmimic" {
        return counts;
    }

    let mut argv = fixture.command(which, shell, scenario, ". \"$NVM_DIR/nvm.sh\"");
    let shell_at = argv.iter().position(|arg| arg == shell.path).unwrap();
    argv.insert(shell_at + 1, "-x".to_string());
    argv.insert(2, "PS4=+ ".to_string());
    let output = Command::new(&argv[0]).args(&argv[1..]).output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stderr.lines().filter_map(|line| line.strip_prefix('+')) {
        let line = line.trim_start_matches('+').trim_start();
        let mut words = line
            .split_whitespace()
            .skip_while(|word| is_assignment(word));
        let mut word = words.next().unwrap_or("");
        if word == "command" {
            word = words.next().unwrap_or("");
        }
        let name = if word.ends_with("/.nvmimic/bin/nvm") || word.ends_with("/.nvmimic/bin/nvmimic")
        {
            "nvmimic"
        } else {
            match word.rsplit('/').next().unwrap_or(word) {
                name @ ("node" | "npm") => name,
                _ => continue,
            }
        };
        *counts.entry(name.to_string()).or_insert(0) += 1;
    }
    counts
}

/// Moves the binary aside and puts a script in its place that logs
/// `nvmimic` and then runs it.
fn wrap_binary(binary: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let real = binary.with_file_name("nvmimic.real");
    fs::rename(binary, &real).map_err(|e| e.to_string())?;
    fs::write(
        binary,
        format!(
            "#!/bin/sh\nprintf 'nvmimic\\n' >> \"$NVMIMIC_LAUNCH_LOG\"\nexec '{}' \"$@\"\n",
            real.display()
        ),
    )
    .map_err(|e| e.to_string())?;
    fs::set_permissions(binary, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())
}

fn is_assignment(word: &str) -> bool {
    word.split_once('=').is_some_and(|(name, _)| {
        !name.is_empty()
            && !name.starts_with(|c: char| c.is_ascii_digit())
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

fn print_table(rows: &[Row]) {
    println!(
        "{:<7} {:<8} {:>8} {:>9} {:>9} {:>10}  launches (nvm | nvmimic)",
        "shell", "scenario", "empty", "nvm", "nvmimic", "nvmimic-Δ"
    );
    for row in rows {
        let nvmimic = row.nvmimic.map_or("-".to_string(), |ms| format!("{ms:.1}"));
        let delta = row
            .nvmimic
            .map_or("-".to_string(), |ms| format!("{:.1}", ms - row.empty));
        println!(
            "{:<7} {:<8} {:>8.1} {:>9.1} {:>9} {:>10}  {} | {}",
            row.shell,
            row.scenario,
            row.empty,
            row.nvm,
            nvmimic,
            delta,
            summarize(&row.nvm_launches),
            row.nvmimic_launches
                .as_ref()
                .map_or("-".to_string(), summarize),
        );
    }
    println!("times are mean milliseconds; nvmimic-Δ is nvmimic minus the empty shell");
}

fn summarize(counts: &BTreeMap<String, usize>) -> String {
    let total: usize = counts.values().sum();
    let parts: Vec<String> = counts
        .iter()
        .map(|(name, n)| format!("{name}×{n}"))
        .collect();
    format!("{total} ({})", parts.join(" "))
}

/// Fails when nvmimic's source time over an empty shell passes the budget in
/// `bench/budget.toml`, or when it starts anything other than its own binary.
fn check_budget(rows: &[Row]) -> ExitCode {
    let budget_file = crate::repo_root().join("bench/budget.toml");
    let text = fs::read_to_string(&budget_file).unwrap_or_default();
    let budget: f64 = text
        .lines()
        .filter_map(|line| line.split_once('='))
        .find(|(key, _)| key.trim() == "source_overhead_ms")
        .and_then(|(_, value)| value.trim().parse().ok())
        .expect("bench/budget.toml needs source_overhead_ms");

    let mut failures = Vec::new();
    for row in rows {
        let delta = row.nvmimic.unwrap() - row.empty;
        if delta > budget {
            failures.push(format!(
                "{} {}: {delta:.1}ms over an empty shell, budget {budget}ms",
                row.shell, row.scenario
            ));
        }
        let launches = row.nvmimic_launches.as_ref().unwrap();
        let foreign: Vec<_> = launches.keys().filter(|name| *name != "nvmimic").collect();
        if !foreign.is_empty() || launches.get("nvmimic").copied().unwrap_or(0) > 1 {
            failures.push(format!(
                "{} {}: launched {}",
                row.shell,
                row.scenario,
                summarize(launches)
            ));
        }
    }
    if failures.is_empty() {
        println!("bench: within budget");
        ExitCode::SUCCESS
    } else {
        for failure in &failures {
            eprintln!("bench: {failure}");
        }
        ExitCode::FAILURE
    }
}

fn shell_quote(arg: &str) -> String {
    if arg
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "-_=./:+,".contains(c))
    {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}
