//! Runs a shell snippet against official nvm and against nvmimic in isolated
//! homes that share one fake mirror, then reports what differs.

use crate::mirror::{self, Mirror};
use crate::subject::Subject;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// One side of a comparison: official nvm, or a program built to match it.
#[derive(Clone, Copy)]
pub enum Impl {
    Nvm,
    Subject(&'static dyn Subject),
}

/// nvmimic as a side of a comparison.
pub const NVMIMIC: Impl = Impl::Subject(&crate::own::Nvmimic);

impl Impl {
    pub fn name(self) -> &'static str {
        match self {
            Impl::Nvm => "nvm",
            Impl::Subject(subject) => subject.name(),
        }
    }

    fn subject(self) -> Option<&'static dyn Subject> {
        match self {
            Impl::Nvm => None,
            Impl::Subject(subject) => Some(subject),
        }
    }
}

impl PartialEq for Impl {
    fn eq(&self, other: &Impl) -> bool {
        self.name() == other.name()
    }
}

impl std::fmt::Debug for Impl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Clone, Debug)]
pub struct Shell {
    pub name: String,
    pub path: PathBuf,
}

impl Shell {
    /// The shell whose syntax this one really speaks. `sh` is bash on macOS
    /// and dash on Debian and Ubuntu.
    pub fn kind(&self) -> &str {
        let is_dash = || {
            fs::canonicalize(&self.path)
                .is_ok_and(|real| real.file_name().is_some_and(|name| name == "dash"))
        };
        if self.name == "sh" && is_dash() {
            "dash"
        } else {
            &self.name
        }
    }
}

pub const SHELL_NAMES: &[&str] = &["bash", "zsh", "sh", "dash"];

/// The shells nvm supports that exist on this machine. `/bin/<name>` is
/// preferred because macOS ships bash 3.2 there.
pub fn shells() -> Vec<Shell> {
    SHELL_NAMES.iter().filter_map(|name| shell(name)).collect()
}

pub fn shell(name: &str) -> Option<Shell> {
    let fixed = Path::new("/bin").join(name);
    let path = if fixed.is_file() {
        fixed
    } else {
        find_on_path(name)?
    };
    Some(Shell {
        name: name.to_string(),
        path,
    })
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|dir| dir.join(name))
            .find(|candidate| candidate.is_file())
    })
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// Where a run keeps what it builds and records: the mirror's files, the
/// recordings of official nvm, the reports. `$NVMIMIC_PARITY_TARGET`, or
/// this repository's `target` folder. Another program that runs the harness
/// from its own repository can point this at its own.
pub fn target_dir() -> PathBuf {
    std::env::var_os("NVMIMIC_PARITY_TARGET")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root().join("target"))
}

/// The nvmimic binary under test: `$NVMIMIC_BIN`, or the workspace's debug build.
pub fn nvmimic_bin() -> PathBuf {
    let path = std::env::var_os("NVMIMIC_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root().join("target/debug/nvmimic"));
    // Not `target_dir()`: this is where cargo put the program.
    assert!(
        path.is_file(),
        "{} is missing, run `cargo build -p nvmimic-cli` first",
        path.display()
    );
    path
}

/// Programs that get a logging shim on `PATH` when launches are counted.
const COUNTED_TOOLS: &[&str] = &[
    "awk",
    "basename",
    "cat",
    "chmod",
    "curl",
    "cut",
    "dirname",
    "find",
    "getconf",
    "grep",
    "head",
    "id",
    "ln",
    "ls",
    "manpath",
    "mkdir",
    "mv",
    "od",
    "openssl",
    "rm",
    "rmdir",
    "sed",
    "sha256sum",
    "shasum",
    "sleep",
    "sort",
    "sw_vers",
    "sysctl",
    "tail",
    "tar",
    "tee",
    "tput",
    "tr",
    "uname",
    "uniq",
    "wget",
    "which",
    "whoami",
    "xargs",
    "xz",
];

/// Names `set` reports whose values change from run to run or describe the
/// shell itself rather than anything nvm did.
const VOLATILE_VARS: &[&str] = &[
    "_",
    "__diff_rc",
    "BASHPID",
    "BASH_ARGC",
    "BASH_ARGV",
    "BASH_COMMAND",
    "BASH_LINENO",
    "BASH_SOURCE",
    "EPOCHREALTIME",
    "EPOCHSECONDS",
    "FUNCNAME",
    "HISTCMD",
    "LINENO",
    "PPID",
    "RANDOM",
    "SECONDS",
    "SRANDOM",
    "TTYIDLE",
    "ZSH_EVAL_CONTEXT",
    "zsh_eval_context",
    "commands",
    "dis_functions",
    "funcfiletrace",
    "funcsourcetrace",
    "funcstack",
    "functions",
    "functions_source",
    "functrace",
    "parameters",
];

#[derive(Debug)]
pub struct Outcome {
    pub stdout: String,
    pub stderr: String,
    pub code: Option<i32>,
    pub timed_out: bool,
    pub state: BTreeMap<String, String>,
    /// Programs launched by the case body, in order, when counting is on.
    pub launches: Vec<String>,
    /// The parts of nvm.sh the case body ran, when tracing is on, see
    /// [`crate::covers`].
    pub covered: BTreeSet<String>,
    /// How long the whole run took, sandbox setup and probe included.
    pub elapsed: Duration,
    /// With `NVMIMIC_PARITY_PHASES` set: how long sourcing nvm.sh took, and
    /// how long sourcing plus the case's setup took.
    pub phases: Phases,
}

/// Time from starting the shell until nvm.sh was sourced, and until the
/// case's setup was done.
pub type Phases = Option<(Duration, Duration)>;

#[derive(Clone, Debug)]
pub struct Options {
    pub timeout: Duration,
    pub count_launches: bool,
    /// Run official nvm from a copy that notes which of its parts run.
    pub trace_covers: bool,
}

impl Default for Options {
    fn default() -> Options {
        let seconds = std::env::var("NVMIMIC_PARITY_TIMEOUT")
            .ok()
            .and_then(|value| value.parse().ok())
            // A network pass downloads real releases the first time.
            .unwrap_or(if crate::mirror::network_pass() {
                900
            } else {
                120
            });
        Options {
            timeout: Duration::from_secs(seconds),
            count_launches: false,
            trace_covers: false,
        }
    }
}

/// What one run does after its sandbox is ready.
struct Run<'a> {
    /// Arguments for `. nvm.sh`, or `None` to leave nvm.sh unsourced.
    source_args: Option<&'a str>,
    setup: &'a str,
    body: &'a str,
    /// Install from the mirror's real-file releases.
    plain_files: bool,
}

pub struct World {
    pub root: PathBuf,
    pub mirror: &'static Mirror,
    pub options: Options,
}

impl Default for World {
    fn default() -> World {
        World::new()
    }
}

impl World {
    pub fn new() -> World {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("nvmimic-diff-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        World {
            root,
            mirror: mirror::shared(),
            options: Options::default(),
        }
    }

    pub fn run(&self, which: Impl, shell: &Shell, body: &str) -> Outcome {
        self.run_with(which, shell, "", "", body)
    }

    /// Runs one case: its body after sourcing nvm.sh, or its test from nvm's
    /// own suite.
    ///
    /// A suite test runs in `suite_side`, a sandbox that is set up once and
    /// then reused for suite tests, the way nvm's own test runner works
    /// through one checkout. The caller keeps one per implementation.
    pub fn run_case(
        &self,
        which: Impl,
        shell: &Shell,
        case: &crate::cases::Case,
        suite_side: &mut SuiteSide,
    ) -> Outcome {
        match &case.suite {
            Some(test) => {
                let side = suite_side.clean(which, &self.options);
                let body = test.command(&shell.path);
                let run = Run {
                    source_args: None,
                    setup: "",
                    body: &body,
                    plain_files: false,
                };
                let mut outcome = self.run_script(which, shell, run, Some(&side));
                suite_side.settle(which, &mut outcome);
                outcome
            }
            None => {
                let source = Some(case.source.as_str());
                let run = Run {
                    source_args: source,
                    setup: &case.setup,
                    body: &case.body,
                    plain_files: case.plain_files,
                };
                self.run_script(which, shell, run, None)
            }
        }
    }

    /// Sources nvm.sh with `source_args`, runs `setup` with its output
    /// discarded, then runs `body`, which is what gets compared. A second run
    /// of the same implementation and shell gets its own side directory.
    pub fn run_with(
        &self,
        which: Impl,
        shell: &Shell,
        source_args: &str,
        setup: &str,
        body: &str,
    ) -> Outcome {
        let run = Run {
            source_args: Some(source_args),
            setup,
            body,
            plain_files: false,
        };
        self.run_script(which, shell, run, None)
    }

    /// `reuse` names a sandbox that is already set up, used in place of a
    /// fresh one.
    fn run_script(&self, which: Impl, shell: &Shell, run: Run, reuse: Option<&Path>) -> Outcome {
        let Run {
            source_args,
            setup,
            body,
            plain_files,
        } = run;
        let network = crate::mirror::network_pass();
        let mirror = match (network, plain_files) {
            (true, _) => format!("{}{}", self.mirror.url, crate::mirror::REAL),
            (false, true) => format!("{}{}", self.mirror.url, crate::mirror::PLAIN),
            (false, false) => self.mirror.url.trim_end_matches('/').to_string(),
        };
        let iojs_mirror = match network {
            true => format!("{}{}", self.mirror.url, crate::mirror::REAL_IOJS),
            false => format!("{mirror}/iojs"),
        };
        let started = Instant::now();
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let side = match reuse {
            Some(side) => side.to_path_buf(),
            None => self
                .root
                .join(format!("{}-{}-{id}", which.name(), shell.name)),
        };
        let home = side.join("home");
        let nvm_dir = home.join(".nvm");
        let bin = side.join("bin");
        let work = side.join("work");
        if reuse.is_none() {
            for dir in [&nvm_dir, &bin, &work] {
                fs::create_dir_all(dir).unwrap();
            }
            install(which, &nvm_dir, &side, &self.options);
        }

        let launch_log = side.join("launches");
        if self.options.count_launches {
            install_shims(&bin);
        }
        let _ = fs::remove_file(&launch_log);

        let state_file = side.join("state");
        let setup = if setup.trim().is_empty() {
            String::new()
        } else {
            format!("{{\n{setup}\n}} >/dev/null 2>&1\n")
        };
        let (count_on, count_off) = if self.options.count_launches {
            (
                format!(
                    "NVMIMIC_LAUNCH_LOG='{}'; export NVMIMIC_LAUNCH_LOG\n",
                    launch_log.display()
                ),
                "unset NVMIMIC_LAUNCH_LOG\n".to_string(),
            )
        } else {
            (String::new(), String::new())
        };
        let source = match source_args {
            Some(args) => format!(". \"$NVM_DIR/nvm.sh\" {args}\n"),
            None => String::new(),
        };
        let covers_log = side.join("covered");
        let _ = fs::remove_file(&covers_log);
        let (trace_on, trace_off) = if self.options.trace_covers {
            let var = crate::covers::LOG_VAR;
            (
                format!("{var}='{}'; export {var}\n", covers_log.display()),
                format!("unset {var}\n"),
            )
        } else {
            (String::new(), String::new())
        };
        // Phase marks change `$?` before the body, so they are opt-in.
        let phases = std::env::var_os("NVMIMIC_PARITY_PHASES").is_some();
        let mark = |name: &str| match phases {
            true => format!(": > \"$DIFF_STATE.{name}\"\n"),
            false => String::new(),
        };
        let script = format!(
            "{source}{}{setup}{}{count_on}{trace_on}{body}\n__diff_rc=$?\n{count_off}{trace_off}{}",
            mark("sourced"),
            mark("ready"),
            probe_script(shell.kind())
        );

        let mut command = Command::new(&shell.path);
        command
            .env_clear()
            .env("HOME", &home)
            .env("NVM_DIR", &nvm_dir)
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin:/usr/sbin:/sbin", bin.display()),
            )
            .env("TERM", "dumb")
            // A login session exports SHELL. Without it bash sets the
            // variable for itself and doesn't export it, and nvmimic then
            // needs a second start to load, so its one-start way of
            // loading would never run here under bash or sh.
            .env("SHELL", &shell.path)
            .env("NVM_NODEJS_ORG_MIRROR", &mirror)
            .env("NVM_IOJS_ORG_MIRROR", &iojs_mirror)
            .env("DIFF_STATE", &state_file)
            .current_dir(&work);
        if let Some(subject) = which.subject() {
            command.envs(subject.env(&side));
        }
        if shell.name == "zsh" {
            command.arg("-f");
        }
        command.arg("-c").arg(script);

        let spawned = std::time::SystemTime::now();
        let (stdout, stderr, code, timed_out) = run_with_timeout(command, self.options.timeout);
        let since_spawn = |name: &str| {
            let mark = PathBuf::from(format!("{}.{name}", state_file.display()));
            let at = fs::metadata(&mark).and_then(|meta| meta.modified()).ok();
            let _ = fs::remove_file(&mark);
            at.and_then(|at| at.duration_since(spawned).ok())
        };
        let phases = since_spawn("sourced").zip(since_spawn("ready"));
        let raw_state = fs::read(&state_file).unwrap_or_default();
        let _ = fs::remove_file(&state_file);

        let mut state = parse_state(&String::from_utf8_lossy(&raw_state), shell.kind());
        for value in state.values_mut() {
            *value = self.normalize(which, value, &side);
        }
        state.extend(
            snapshot(which, &home, &work)
                .into_iter()
                .map(|(key, value)| (key, self.normalize(which, &value, &side))),
        );
        if let Some(subject) = which.subject() {
            hide_functions(&mut state, &subject.own_functions());
        }

        let covered = fs::read_to_string(&covers_log)
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect();
        let launches = fs::read_to_string(&launch_log)
            .unwrap_or_default()
            .lines()
            .map(|line| self.normalize(which, line, &side))
            .collect();

        Outcome {
            stdout: self.normalize(which, &String::from_utf8_lossy(&stdout), &side),
            stderr: self.normalize(which, &String::from_utf8_lossy(&stderr), &side),
            code,
            timed_out,
            state,
            launches,
            covered,
            elapsed: started.elapsed(),
            phases,
        }
    }

    /// Replaces sandbox-specific paths so the two sides can be compared.
    fn normalize(&self, which: Impl, text: &str, side: &Path) -> String {
        let side = side.display().to_string();
        let text = text
            .replace(&format!("{side}/home/.nvm/versions/node/"), "<install>/")
            .replace(&format!("{side}/home"), "<home>")
            .replace(&format!("{side}/bin"), "<bin>")
            .replace(&side, "<side>")
            .replace(&crate::guard::stage_dir().display().to_string(), "<stage>")
            .replace(self.mirror.url.trim_end_matches('/'), "<mirror>")
            .replace(&self.mirror.root.display().to_string(), "<mirror-files>")
            // macOS reports temp paths both with and without the /private prefix.
            .replace("/private<", "<");
        let text = match which.subject() {
            Some(subject) => subject.normalize(text),
            None => text,
        };
        match crate::mirror::network_pass() {
            true => settle_progress(&text),
            false => text,
        }
    }
}

/// curl redraws its progress bar as a download goes, and how often depends
/// on how fast the bytes arrive. Over the real network the two sides never
/// redraw alike, so a line of redraws is cut down to the last one.
fn settle_progress(text: &str) -> String {
    // Two styles: `####   23.9%` when curl knows the size, and a moving
    // `-=O=-  #  #` while it does not.
    let is_bar = |part: &str| {
        part.chars()
            .all(|c| "#=O- .%".contains(c) || c.is_ascii_digit())
    };
    let mut lines: Vec<String> = Vec::new();
    for line in text.split('\n') {
        let parts: Vec<&str> = line
            .split('\r')
            .filter(|part| !part.trim().is_empty())
            .collect();
        // Real npm ends an install with how long it took.
        if let Some((before, took)) = line.rsplit_once(" in ")
            && let Some(seconds) = took.strip_suffix('s')
            && !seconds.is_empty()
            && seconds.chars().all(|c| c.is_ascii_digit() || c == '.')
        {
            lines.push(format!("{before} in <time>s"));
            continue;
        }
        match parts.last() {
            // A redraw that ended in a line break of its own leaves a line
            // holding nothing but the return to its start.
            None if line.contains('\r') => {}
            Some(last) if line.contains('\r') && parts.iter().all(|part| is_bar(part)) => {
                lines.push(last.to_string())
            }
            _ => lines.push(line.to_string()),
        }
    }
    lines.join("\n")
}

/// Runs official nvm through a case's sourcing and setup and stops there. What
/// this leaves behind is where the case's body starts from, so comparing a
/// full run against it shows what the body itself did.
pub fn before_body(shell: &Shell, case: &crate::cases::Case) -> Outcome {
    let (source_args, setup) = match &case.suite {
        // A suite test brings its own sourcing and has no setup.
        Some(_) => (None, ""),
        None => (Some(case.source.as_str()), case.setup.as_str()),
    };
    let run = Run {
        source_args,
        setup,
        body: ":",
        plain_files: case.plain_files,
    };
    World::new().run_script(Impl::Nvm, shell, run, None)
}

/// A sandbox for nvm's own suite that is used for one test after another.
///
/// Every test has to start from the same files, or its outcome would depend
/// on which tests ran before it. So after each test the sandbox is compared
/// with a fresh one. Most tests clean up after themselves and the sandbox is
/// used again. When a test leaves something behind, the next test gets a new
/// sandbox, and the old one stays on disk until a later run sweeps it.
pub struct SuiteSide {
    root: PathBuf,
    label: String,
    generation: u32,
    changed: bool,
    used: u32,
}

impl SuiteSide {
    pub fn new(root: &Path, label: &str) -> SuiteSide {
        SuiteSide {
            root: root.to_path_buf(),
            label: label.to_string(),
            generation: 0,
            changed: false,
            used: 0,
        }
    }

    /// How many sandboxes this side has gone through.
    pub fn used(&self) -> u32 {
        self.used
    }

    /// A sandbox no test has left anything in, set up for `which`.
    fn clean(&mut self, which: Impl, options: &Options) -> PathBuf {
        if self.changed {
            self.generation += 1;
            self.changed = false;
        }
        if self.used <= self.generation {
            self.used = self.generation + 1;
        }
        let side = self
            .root
            .join(format!("{}-{}", self.label, self.generation));
        prepare_suite_side(which, &side, options);
        side
    }

    /// Reduces the files in a suite test's outcome to those that differ from
    /// a fresh sandbox, and notes whether there were any.
    fn settle(&mut self, which: Impl, outcome: &mut Outcome) {
        let fresh = fresh_suite_files(which);
        let gone: Vec<String> = fresh
            .keys()
            .filter(|key| !outcome.state.contains_key(*key))
            .cloned()
            .collect();
        outcome
            .state
            .retain(|key, value| !key.starts_with("fs:") || fresh.get(key) != Some(value));
        outcome
            .state
            .extend(gone.into_iter().map(|key| (key, "missing".to_string())));
        self.changed = outcome.state.keys().any(|key| key.starts_with("fs:"));
    }
}

/// The files of a suite sandbox before any test has run in it.
fn fresh_suite_files(which: Impl) -> &'static BTreeMap<String, String> {
    static FRESH: std::sync::Mutex<BTreeMap<&'static str, &'static BTreeMap<String, String>>> =
        std::sync::Mutex::new(BTreeMap::new());
    let mut fresh = FRESH.lock().unwrap();
    fresh.entry(which.name()).or_insert_with(|| {
        let world = World::new();
        let side = world.root.join("fresh");
        prepare_suite_side(which, &side, &world.options);
        let files = snapshot(which, &side.join("home"), &side.join("work"))
            .into_iter()
            .map(|(key, value)| (key, world.normalize(which, &value, &side)))
            .collect();
        Box::leak(Box::new(files))
    })
}

// A `World` leaves its directory behind on purpose. macOS scans new files
// the first time they run, and deleting a sandbox while scans of its files
// are still queued is part of what crashed the scanning service in
// an earlier run. `guard::sweep_stale` removes old sandboxes at the start of a
// later run instead.

/// Sets up a reusable sandbox for nvm's own test suite, once: the
/// implementation in `$NVM_DIR`, and a copy of the suite at `$NVM_DIR/test`.
pub fn prepare_suite_side(which: Impl, side: &Path, options: &Options) {
    let nvm_dir = side.join("home/.nvm");
    if nvm_dir.join("test").is_dir() {
        return;
    }
    // The suite's own hooks leave empty `src` and `alias` directories after
    // every test, so a sandbox starts out with them.
    for dir in [
        nvm_dir.join("src"),
        nvm_dir.join("alias"),
        side.join("bin"),
        side.join("work"),
    ] {
        fs::create_dir_all(dir).unwrap();
    }
    let options = Options {
        count_launches: false,
        ..options.clone()
    };
    install(which, &nvm_dir, side, &options);
    copy_tree(&crate::suite::tree(), &nvm_dir.join("test"));

    // The suite's mock programs get run, so each sandbox links to one shared
    // copy of them, which macOS then scans only once.
    let staged = staged(Path::new("suite-mocks"), |staged| {
        copy_tree(&crate::suite::tree().join("mocks"), staged)
    });
    let mocks = nvm_dir.join("test/mocks");
    for entry in fs::read_dir(&staged).unwrap().flatten() {
        let target = mocks.join(entry.file_name());
        let _ = fs::remove_file(&target);
        fs::hard_link(entry.path(), target).unwrap();
    }
}

/// A file or directory made once per process under the stage directory.
/// Sandboxes link to it, so a script many sandboxes run is one file to macOS.
fn staged(name: &Path, make: impl FnOnce(&Path)) -> PathBuf {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _held = LOCK.lock().unwrap();
    let path = crate::guard::stage_dir().join(name);
    if fs::symlink_metadata(&path).is_err() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        make(&path);
    }
    path
}

/// The script that runs one test of nvm's suite, see [`crate::suite::RUNNER`].
pub fn suite_runner() -> PathBuf {
    staged(Path::new("run-nvm-test"), |path| {
        fs::write(path, crate::suite::RUNNER).unwrap()
    })
}

/// Puts one implementation into `$NVM_DIR` the way its installer would.
fn install(which: Impl, nvm_dir: &Path, side: &Path, options: &Options) {
    let count_launches = options.count_launches;
    block_sudo(side);
    match which {
        Impl::Nvm => {
            let reference = crate::reference_dir();
            copy(
                &reference.join("bash_completion"),
                &nvm_dir.join("bash_completion"),
            );
            if options.trace_covers {
                let source = fs::read_to_string(reference.join("nvm.sh")).unwrap();
                fs::write(nvm_dir.join("nvm.sh"), crate::covers::traced(&source)).unwrap();
            } else {
                copy(&reference.join("nvm.sh"), &nvm_dir.join("nvm.sh"));
            }
            link_staged("nvm/nvm-exec", &reference.join("nvm-exec"), nvm_dir);
        }
        Impl::Subject(subject) => subject.install(nvm_dir, side, count_launches),
    }
}

/// Takes a subject's own functions out of the list of functions a run left
/// behind. Every other name in that list is compared with official nvm's.
fn hide_functions(state: &mut BTreeMap<String, String>, own: &[String]) {
    if let Some(functions) = state.get_mut("functions") {
        *functions = functions
            .split(' ')
            .filter(|name| !own.iter().any(|own| own == name))
            .collect::<Vec<_>>()
            .join(" ");
    }
}

/// Puts a `sudo` that always fails first on the sandbox's `PATH`. Two of
/// nvm's own tests call `sudo`. The real one stops to ask for a password or
/// a fingerprint, which holds a worker until the timeout and puts a prompt
/// on the screen of whoever is at the machine. No test gets to run anything
/// as root from here.
///
/// Each attempt adds a line to `target/parity-sudo.log`: the time, the
/// folder it came from, and what it asked `sudo` to do. This only
/// catches `sudo` looked up by name. A test that changes `PATH` or spells
/// out `/usr/bin/sudo` gets past it.
fn block_sudo(side: &Path) {
    let sudo = staged(Path::new("sudo"), |path| {
        let log = target_dir().join("parity-sudo.log");
        write_script(
            path,
            &format!(
                "#!/bin/sh\nprintf '%s\\t%s\\tsudo %s\\n' \"$(/bin/date '+%F %T')\" \"$PWD\" \"$*\" >> '{}'\necho 'sudo: not available in a parity run' >&2\nexit 1\n",
                log.display()
            ),
        )
    });
    fs::hard_link(sudo, side.join("bin/sudo")).unwrap();
}

/// Puts `nvm-exec` into `$NVM_DIR` as a hard link to one staged copy. It is
/// the one script of nvm's that gets run rather than sourced.
pub(crate) fn link_staged(name: &str, source: &Path, nvm_dir: &Path) {
    let staged = staged(Path::new(name), |staged| copy(source, staged));
    fs::hard_link(staged, nvm_dir.join("nvm-exec")).unwrap();
}

fn install_shims(bin: &Path) {
    let shims = staged(Path::new("shims"), |shims| {
        fs::create_dir_all(shims).unwrap();
        for tool in COUNTED_TOOLS {
            let Some(real) = ["/usr/bin", "/bin", "/usr/sbin", "/sbin"]
                .iter()
                .map(|dir| Path::new(dir).join(tool))
                .find(|path| path.is_file())
            else {
                continue;
            };
            write_script(
                &shims.join(tool),
                &format!(
                    "#!/bin/sh\n[ -z \"$NVMIMIC_LAUNCH_LOG\" ] || printf '%s\\n' {tool} >> \"$NVMIMIC_LAUNCH_LOG\"\nexec '{}' \"$@\"\n",
                    real.display()
                ),
            );
        }
    });
    for entry in fs::read_dir(shims).unwrap().flatten() {
        let target = bin.join(entry.file_name());
        if fs::symlink_metadata(&target).is_err() {
            fs::hard_link(entry.path(), target).unwrap();
        }
    }
}

/// Runs the command in its own process group so a timeout can stop
/// everything it started, including a `sleep` inside nvm's install lock.
fn run_with_timeout(
    mut command: Command,
    timeout: Duration,
) -> (Vec<u8>, Vec<u8>, Option<i32>, bool) {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    let mut child = command.spawn().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out = thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stdout.read_to_end(&mut buffer);
        buffer
    });
    let err = thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stderr.read_to_end(&mut buffer);
        buffer
    });

    let deadline = Instant::now() + timeout;
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            timed_out = true;
            let _ = Command::new("/bin/kill")
                .args(["-KILL", &format!("-{}", child.id())])
                .status();
            break child.wait().unwrap();
        }
        thread::sleep(Duration::from_millis(10));
    };
    (
        out.join().unwrap(),
        err.join().unwrap(),
        status.code(),
        timed_out,
    )
}

/// Probes what the case left behind in the shell. Each record is separated
/// by an ASCII record separator so multi-line values survive.
fn probe_script(shell: &str) -> String {
    let list_vars = match shell {
        "bash" | "sh" => "(set -o posix; set)",
        _ => "set",
    };
    let list_functions = match shell {
        "bash" | "sh" => "compgen -A function",
        "zsh" => "print -rl -- ${(k)functions}",
        _ => ":",
    };
    let mut script = String::from("{\n  printf 'rc=%s\\036' \"$__diff_rc\"\n");
    script.push_str(&format!(
        "  printf 'vars\\n'; {list_vars}; printf '\\036'\n"
    ));
    script.push_str(&format!(
        "  printf 'functions\\n'; {list_functions}; printf '\\036'\n"
    ));
    script.push_str("  printf 'defined\\n'\n");
    for name in crate::nvm_function_names() {
        script.push_str(&format!(
            "  if type {name} >/dev/null 2>&1; then printf '{name}\\n'; fi\n"
        ));
    }
    script.push_str("  printf '\\036'\n");
    script.push_str("  printf 'node=%s\\036' \"$(command -v node 2>/dev/null)\"\n");
    script.push_str("} > \"$DIFF_STATE\"\nexit \"$__diff_rc\"\n");
    script
}

fn parse_state(raw: &str, shell: &str) -> BTreeMap<String, String> {
    let mut state = BTreeMap::new();
    for record in raw.split('\u{1e}').filter(|record| !record.is_empty()) {
        if let Some(rest) = record.strip_prefix("vars\n") {
            state.extend(parse_vars(rest, shell));
        } else if let Some(rest) = record.strip_prefix("functions\n") {
            if matches!(shell, "bash" | "sh" | "zsh") {
                let names: BTreeSet<&str> = rest.lines().collect();
                state.insert(
                    "functions".to_string(),
                    names.into_iter().collect::<Vec<_>>().join(" "),
                );
            }
        } else if let Some(rest) = record.strip_prefix("defined\n") {
            state.insert(
                "defined".to_string(),
                rest.lines().collect::<Vec<_>>().join(" "),
            );
        } else if let Some((key, value)) = record.split_once('=') {
            state.insert(key.to_string(), value.to_string());
        }
    }
    state
}

/// Splits `set` output into one entry per variable. A line starting with
/// `NAME=` opens a new variable, and any other line continues the previous
/// value, which keeps quoted multi-line values together.
///
/// zsh writes a value with a newline in it on one line, and lists some of
/// its own parameters as a bare name with no value. Those bare names are
/// dropped, since they would otherwise read as part of the variable before
/// them.
fn parse_vars(listing: &str, shell: &str) -> BTreeMap<String, String> {
    let mut vars: BTreeMap<String, String> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in listing.lines() {
        let name = line.split_once('=').map(|(name, _)| name).filter(|name| {
            !name.is_empty()
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !name.starts_with(|c: char| c.is_ascii_digit())
        });
        match name {
            Some(name) => {
                current = Some(name.to_string());
                vars.insert(format!("var:{name}"), line[name.len() + 1..].to_string());
            }
            None if shell == "zsh" => {}
            None => {
                if let Some(value) = current
                    .as_ref()
                    .and_then(|name| vars.get_mut(&format!("var:{name}")))
                {
                    value.push('\n');
                    value.push_str(line);
                }
            }
        }
    }
    vars.retain(|key, _| !VOLATILE_VARS.contains(&&key[4..]));
    vars
}

/// Everything under the case's home and working directories, except the
/// implementation's own scripts. Small text files are recorded whole so a
/// difference is readable, anything else by size and hash.
fn snapshot(which: Impl, home: &Path, work: &Path) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    for (label, root) in [("home", home), ("work", work)] {
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let relative = path.strip_prefix(root).unwrap().display().to_string();
                if label == "home"
                    && (is_implementation_file(&relative)
                        || which
                            .subject()
                            .is_some_and(|subject| subject.ignores_file(&relative)))
                {
                    continue;
                }
                // Real npm keeps a download cache and the time of its last
                // update check, both different on every run.
                if label == "home"
                    && crate::mirror::network_pass()
                    && [".npm", ".config/configstore"]
                        .iter()
                        .any(|own| Path::new(&relative).starts_with(own))
                {
                    continue;
                }
                let key = format!("fs:{label}/{relative}");
                let Ok(meta) = fs::symlink_metadata(&path) else {
                    continue;
                };
                let mode = meta.permissions().mode() & 0o7777;
                if meta.file_type().is_symlink() {
                    let target = fs::read_link(&path).unwrap_or_default();
                    files.insert(key, format!("link -> {}", target.display()));
                } else if meta.is_dir() {
                    files.insert(key, format!("dir {mode:o}"));
                    stack.push(path);
                } else {
                    // Real npm writes the temp folder it unpacked a package
                    // in, named after its process id, into the package's
                    // own package.json.
                    if crate::mirror::network_pass()
                        && path.file_name().is_some_and(|name| name == "package.json")
                        && relative.contains("/node_modules/")
                    {
                        files.insert(key, format!("file {mode:o}"));
                        continue;
                    }
                    let bytes = fs::read(&path).unwrap_or_default();
                    let summary = match std::str::from_utf8(&bytes) {
                        Ok(text) if bytes.len() <= 4096 => format!("file {mode:o}\n{text}"),
                        _ => format!(
                            "file {mode:o} {} bytes sha256 {}",
                            meta.size(),
                            hex(&Sha256::digest(&bytes))
                        ),
                    };
                    files.insert(key, summary);
                }
            }
        }
    }
    files
}

fn is_implementation_file(relative: &str) -> bool {
    matches!(
        relative,
        ".nvm/nvm.sh" | ".nvm/nvm-exec" | ".nvm/bash_completion"
    )
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn copy(from: &Path, to: &Path) {
    fs::copy(from, to).unwrap();
}

/// Copies a directory, as a copy-on-write clone where the filesystem has one.
fn copy_tree(from: &Path, to: &Path) {
    let flag = if cfg!(target_os = "macos") {
        "-cR"
    } else {
        "-R"
    };
    let status = Command::new("cp").arg(flag).arg(from).arg(to).status();
    assert!(
        status.is_ok_and(|status| status.success()),
        "copying {} failed",
        from.display()
    );
}

pub(crate) fn write_script(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// A readable list of everything that differs between two outcomes.
pub fn diff(left: (&str, &Outcome), right: (&str, &Outcome)) -> Vec<String> {
    differences(left, right)
        .into_iter()
        .map(|(_, text)| text)
        .collect()
}

/// Everything that differs between two outcomes, each with its key: `timed
/// out`, `exit`, `stdout`, `stderr`, or the name of a piece of what the run
/// left behind, such as `var:NVM_BIN` or `fs:home/.nvm/alias/default`.
pub fn differences(left: (&str, &Outcome), right: (&str, &Outcome)) -> Vec<(String, String)> {
    let (left_name, a) = left;
    let (right_name, b) = right;
    let mut differences = Vec::new();

    if a.timed_out || b.timed_out {
        differences.push((
            "timed out".to_string(),
            format!(
                "timed out: {left_name} {}, {right_name} {}",
                a.timed_out, b.timed_out
            ),
        ));
    }
    if a.code != b.code {
        differences.push((
            "exit".to_string(),
            format!("exit: {left_name} {:?}, {right_name} {:?}", a.code, b.code),
        ));
    }
    for (name, x, y) in [
        ("stdout", &a.stdout, &b.stdout),
        ("stderr", &a.stderr, &b.stderr),
    ] {
        if x != y {
            differences.push((
                name.to_string(),
                format!("{name}:\n--- {left_name}\n{x}--- {right_name}\n{y}---"),
            ));
        }
    }
    let keys: BTreeSet<_> = a.state.keys().chain(b.state.keys()).collect();
    for key in keys {
        let x = a.state.get(key);
        let y = b.state.get(key);
        if x != y {
            differences.push((
                key.clone(),
                format!("{key}: {left_name} {x:?}, {right_name} {y:?}"),
            ));
        }
    }
    differences
}
