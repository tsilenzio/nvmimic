//! A local stand-in for nodejs.org/dist that both real nvm and nvmimic
//! install from. Each release is a tiny tree whose `node` and `npm` are shell
//! scripts, packed the way the real dist directory packs them.
//!
//! The mirror holds every release twice. Under `/` the two scripts are
//! packed as symlinks to one shared, read-only copy per version. macOS scans
//! each new file the first time it is run, and a full parity run installs
//! Node thousands of times. With real files that was thousands of scans,
//! enough to bring the scanning service down. A symlink to a file that was
//! already scanned costs nothing.
//!
//! Under `/plain` the scripts are real files, the way nodejs.org packs them.
//! Cases that change an installed `node` or `npm` install from there, since
//! writing through a symlink would change the shared copy for everyone.
//! [`Mirror::verify`] checks after a run that the shared copies are untouched.
//!
//! Every release also has a source archive, for `nvm install -s`. Its
//! `configure` and `Makefile` are stand-ins: configuring notes where to
//! install, and `make install` copies a ready tree there. A case can make
//! either step fail by exporting `NVMIMIC_TEST_BUILD_FAILS=configure` or
//! `=make`.
//!
//! In a network pass (`cargo xtask parity --network`) the mirror also
//! answers under `/real` and `/real-iojs` with the files nodejs.org and
//! iojs.org serve, fetched once each and kept in `target/parity-network/`.
//!
//! The files are built once and kept in `target/parity-mirror/`, so every
//! run serves the very same archives. Recorded outcomes of real nvm (see
//! [`crate::golden`]) hold the hashes of downloaded archives, and those only
//! stay true while the archives do.

use sha2::{Digest, Sha256};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

pub struct Release {
    pub version: &'static str,
    pub lts: Option<&'static str>,
    pub date: &'static str,
}

pub const RELEASES: &[Release] = &[
    Release {
        version: "26.1.0",
        lts: None,
        date: "2026-09-01",
    },
    Release {
        version: "24.21.0",
        lts: Some("Krypton"),
        date: "2026-08-20",
    },
    Release {
        version: "24.15.0",
        lts: Some("Krypton"),
        date: "2026-03-10",
    },
    Release {
        version: "22.20.0",
        lts: Some("Jod"),
        date: "2026-07-15",
    },
    Release {
        version: "20.20.2",
        lts: Some("Iron"),
        date: "2026-06-30",
    },
    Release {
        version: "18.20.8",
        lts: Some("Hydrogen"),
        date: "2025-03-27",
    },
    Release {
        version: "14.21.3",
        lts: Some("Fermium"),
        date: "2023-02-16",
    },
];

const NPM_VERSION: &str = "10.9.0";

/// The path under the mirror whose releases hold real files.
pub const PLAIN: &str = "plain";

/// nodejs.org has no darwin-arm64 builds before Node 16.
fn platforms(version: &str) -> &'static [&'static str] {
    let major: u32 = version.split('.').next().unwrap().parse().unwrap();
    if major < 16 {
        &["darwin-x64", "linux-x64", "linux-arm64"]
    } else {
        &["darwin-arm64", "darwin-x64", "linux-x64", "linux-arm64"]
    }
}

/// The `files` column of index.tab names builds the way nodejs.org does.
fn files(version: &str) -> Vec<String> {
    platforms(version)
        .iter()
        .map(|platform| match platform.strip_prefix("darwin-") {
            Some(arch) => format!("osx-{arch}-tar"),
            None => platform.to_string(),
        })
        .chain(["src".to_string()])
        .collect()
}

pub struct Mirror {
    pub url: String,
    /// The directory holding everything the mirror serves or links to.
    pub root: PathBuf,
    pub dist: PathBuf,
    pub requests: Arc<Mutex<Vec<String>>>,
    /// A hash over every file of the mirror. It changes when the mirror is
    /// built again, even from the same recipe.
    pub identity: String,
    /// The shared scripts with the hash and mode each was built with.
    shared: Vec<(PathBuf, String, u32)>,
}

static SHARED: OnceLock<Mirror> = OnceLock::new();

/// The name of the file listing everything in a finished mirror directory.
const MANIFEST: &str = "manifest";

/// One mirror per test process, serving the files kept in
/// `target/parity-mirror/`.
pub fn shared() -> &'static Mirror {
    SHARED.get_or_init(|| {
        crate::guard::sweep_stale();
        Mirror::start(&kept_files())
    })
}

/// The directory with the mirror's files, built first if it is missing or
/// was built by a different version of this file.
fn kept_files() -> PathBuf {
    let recipe: String = Sha256::digest(include_str!("mirror.rs"))
        .iter()
        .take(6)
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let parent = crate::harness::target_dir().join("parity-mirror");
    let root = parent.join(&recipe);
    if intact(&root) {
        return root;
    }
    if fs::symlink_metadata(&root).is_ok() {
        // Moved aside instead of deleted: a sandbox from an earlier run may
        // still link into it.
        let aside = parent.join(format!("{recipe}.broken-{}", std::process::id()));
        fs::rename(&root, aside).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    build_dist(
        &root.join("dist"),
        &root.join("linked"),
        Some(&root.join("shared")),
    );
    build_dist(&root.join("dist").join(PLAIN), &root.join("plain"), None);
    fs::write(root.join(MANIFEST), manifest(&root)).unwrap();
    root
}

/// One line per file the mirror serves or shares: path, hash and mode.
fn manifest(root: &Path) -> String {
    let mut files = Vec::new();
    let mut pending = vec![root.join("dist"), root.join("shared")];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            match fs::symlink_metadata(&path) {
                Ok(meta) if meta.is_dir() => pending.push(path),
                Ok(meta) if meta.is_file() => files.push(path),
                _ => {}
            }
        }
    }
    files.sort();
    files
        .iter()
        .map(|path| {
            let relative = path.strip_prefix(root).unwrap().display();
            format!("{relative}\t{}\t{:o}\n", sha256(path), mode_of(path))
        })
        .collect()
}

/// Whether a mirror directory is complete and still holds what it was built
/// with.
fn intact(root: &Path) -> bool {
    fs::read_to_string(root.join(MANIFEST)).is_ok_and(|listed| listed == manifest(root))
}

impl Mirror {
    pub fn start(root: &Path) -> Mirror {
        let dist = root.join("dist");
        let identity = sha256(&root.join(MANIFEST));
        let mut shared = Vec::new();
        for release in RELEASES {
            let top = root.join("shared").join(format!("v{}", release.version));
            for script in SCRIPTS.into_iter().chain([CONFIGURE]) {
                let path = top.join(script);
                let mode = mode_of(&path);
                shared.push((path.clone(), sha256(&path), mode));
            }
        }
        let served = dist.clone();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::clone(&requests);

        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let dist = served.clone();
                let log = Arc::clone(&log);
                thread::spawn(move || serve(stream, &dist, &log));
            }
        });

        Mirror {
            url,
            root: root.to_path_buf(),
            dist,
            requests,
            identity,
            shared,
        }
    }

    /// Checks that no case changed a shared script. A case that needs to
    /// change an installed `node` or `npm` has to install from `/plain`.
    pub fn verify(&self) -> Result<(), String> {
        for (path, hash, mode) in &self.shared {
            if !path.is_file() || &sha256(path) != hash || mode_of(path) != *mode {
                return Err(format!(
                    "a case changed {}, which every sandbox shares. Give that case a \
                     `# mirror: plain` header so it installs real files instead.",
                    path.display()
                ));
            }
        }
        Ok(())
    }

    pub fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}

fn serve(mut stream: TcpStream, dist: &Path, log: &Mutex<Vec<String>>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    let mut header = String::new();
    while reader.read_line(&mut header).is_ok_and(|n| n > 2) {
        header.clear();
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("/").to_string();
    let path = target
        .split('?')
        .next()
        .unwrap_or("/")
        .trim_start_matches('/');
    log.lock().unwrap().push(format!("{method} /{path}"));

    let file = match real_origin(path) {
        Some(url) if network_pass() => fetched(&url),
        _ => dist.join(path),
    };
    let (code, reason, body) = if !path.contains("..") && file.is_file() {
        (200, "OK", fs::read(&file).unwrap())
    } else {
        (404, "Not Found", b"not found\n".to_vec())
    };

    let head = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    if method != "HEAD" {
        let _ = stream.write_all(&body);
    }
}

/// Whether this is a network pass, where nvm's tests that install real Node
/// releases run against what nodejs.org serves.
pub fn network_pass() -> bool {
    std::env::var_os("NVMIMIC_PARITY_NETWORK").is_some()
}

/// The path under the mirror that stands for nodejs.org/dist.
pub const REAL: &str = "real";
/// The same for iojs.org/dist.
pub const REAL_IOJS: &str = "real-iojs";

/// The address on the internet that a path under `/real` or `/real-iojs`
/// stands for.
fn real_origin(path: &str) -> Option<String> {
    if let Some(rest) = path.strip_prefix("real-iojs/") {
        return Some(format!("https://iojs.org/dist/{rest}"));
    }
    path.strip_prefix("real/")
        .map(|rest| format!("https://nodejs.org/dist/{rest}"))
}

/// Where a file from the internet is kept, fetching it with curl the first
/// time it is asked for. One download at a time. An address that has no
/// file is remembered too, so it is asked for once.
fn fetched(url: &str) -> PathBuf {
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
    let _turn = ONE_AT_A_TIME.lock().unwrap();
    let kept = crate::harness::target_dir()
        .join("parity-network")
        .join(url.trim_start_matches("https://"));
    let missing = kept.with_extension("nvmimic-missing");
    if kept.is_file() || missing.is_file() || url.contains("..") {
        return kept;
    }
    fs::create_dir_all(kept.parent().unwrap()).unwrap();
    let partial = kept.with_extension("nvmimic-partial");
    let status = Command::new("curl")
        .args(["--fail", "--silent", "--show-error", "--location"])
        .args(["--max-time", "900", "--output"])
        .arg(&partial)
        .arg(url)
        .status();
    if status.is_ok_and(|status| status.success()) {
        fs::rename(&partial, &kept).unwrap();
    } else {
        let _ = fs::remove_file(&partial);
        fs::write(&missing, "").unwrap();
    }
    kept
}

/// The stand-in for Node's `configure` in a source archive.
const CONFIGURE: &str = "configure";

const CONFIGURE_SCRIPT: &str = r#"#!/bin/sh
echo "configure: $*"
if [ "${NVMIMIC_TEST_BUILD_FAILS-}" = configure ]; then
  echo "configure: failing because the test asked" >&2
  exit 3
fi
for option in "$@"; do
  case "$option" in
    --prefix=*) echo "PREFIX = ${option#--prefix=}" > config.mk ;;
  esac
done
"#;

/// The stand-in for Node's `Makefile`. Building prints the compilers it was
/// given, and installing copies the ready tree packed beside it.
const MAKEFILE: &str = "include config.mk\n\
\n\
all:\n\
\t@echo \"make: building with CC=$(CC) CXX=$(CXX)\"\n\
\t@if [ \"$$NVMIMIC_TEST_BUILD_FAILS\" = make ]; then echo 'make: failing because the test asked' >&2; exit 2; fi\n\
\n\
install: all\n\
\t@echo \"make: installing\"\n\
\t@mkdir -p \"$(PREFIX)\"\n\
\t@cp -R tree/. \"$(PREFIX)/\"\n";

const INDEX_HEADER: &str =
    "version\tdate\tfiles\tnpm\tv8\tuv\tzlib\topenssl\tmodules\tlts\tsecurity\n";

/// The two files of a release that get run.
const SCRIPTS: [&str; 2] = ["bin/node", "lib/node_modules/npm/bin/npm-cli.js"];

fn mode_of(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).map_or(0, |meta| meta.permissions().mode() & 0o7777)
}

/// Builds one dist directory. With `shared`, each release's scripts are
/// symlinks into that directory, which gets one real tree per version.
fn build_dist(dist: &Path, scratch: &Path, shared: Option<&Path>) {
    fs::create_dir_all(dist).unwrap();
    let mut tab = String::from(INDEX_HEADER);
    let mut json = Vec::new();

    for release in RELEASES {
        let version = format!("v{}", release.version);
        let dir = dist.join(&version);
        fs::create_dir_all(&dir).unwrap();

        let shared = shared.map(|shared| {
            let shared = shared.join(&version);
            build_tree(&shared, &version, None);
            // Read-only, so a case that writes through a symlink fails
            // instead of changing what every other sandbox runs.
            write_script(&shared.join(CONFIGURE), CONFIGURE_SCRIPT);
            for script in SCRIPTS.into_iter().chain([CONFIGURE]) {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(shared.join(script), fs::Permissions::from_mode(0o555))
                    .unwrap();
            }
            shared
        });

        let mut sums = String::new();
        let source = format!("node-{version}");
        let builds = platforms(release.version)
            .iter()
            .map(|platform| format!("node-{version}-{platform}"));
        for name in builds.chain([source.clone()]) {
            let tree = scratch.join("trees").join(&name);
            let top = tree.join(&name);
            if name == source {
                build_tree(&top.join("tree"), &version, shared.as_deref());
                fs::write(top.join("Makefile"), MAKEFILE).unwrap();
                match &shared {
                    Some(shared) => {
                        std::os::unix::fs::symlink(shared.join(CONFIGURE), top.join(CONFIGURE))
                            .unwrap()
                    }
                    None => write_script(&top.join(CONFIGURE), CONFIGURE_SCRIPT),
                }
            } else {
                build_tree(&top, &version, shared.as_deref());
            }

            for (ext, flag) in [("tar.gz", "-czf"), ("tar.xz", "-cJf")] {
                let archive = dir.join(format!("{name}.{ext}"));
                let status = Command::new("tar")
                    .args([flag, &archive.display().to_string(), "-C"])
                    .arg(&tree)
                    .arg(&name)
                    .status()
                    .unwrap();
                assert!(status.success(), "tar failed for {name}.{ext}");
                sums.push_str(&format!("{}  {name}.{ext}\n", sha256(&archive)));
            }
        }
        fs::write(dir.join("SHASUMS256.txt"), sums).unwrap();

        // nodejs.org writes `false` where a release has no LTS name.
        json.push(serde_json::json!({
            "version": version,
            "date": release.date,
            "files": files(release.version),
            "npm": NPM_VERSION,
            "lts": release.lts.map_or(serde_json::Value::Bool(false), Into::into),
            "security": false,
        }));
        let lts = release.lts.unwrap_or("-");
        tab.push_str(&format!(
            "{version}\t{}\t{}\t{NPM_VERSION}\t-\t-\t-\t-\t-\t{lts}\t-\n",
            release.date,
            files(release.version).join(",")
        ));
    }

    fs::write(dist.join("index.tab"), tab).unwrap();
    // nvm reads `index.tab`. Other tools that install Node read the same
    // list as `index.json`, so the mirror has both, as nodejs.org does.
    let json = serde_json::to_string_pretty(&json).unwrap();
    fs::write(dist.join("index.json"), format!("{json}\n")).unwrap();

    // nvm also lists io.js releases. An empty index keeps it off iojs.org.
    fs::create_dir_all(dist.join("iojs")).unwrap();
    fs::write(dist.join("iojs/index.tab"), INDEX_HEADER).unwrap();
    fs::write(dist.join("iojs/index.json"), "[]\n").unwrap();
}

/// The layout nvm's post-install validation checks: an executable
/// `bin/node` and a `bin/npm` symlink that resolves. Both scripts note each
/// launch in `$NVMIMIC_LAUNCH_LOG` when the harness counts launches. With
/// `shared`, the two scripts are symlinks to the ones in that tree.
fn build_tree(top: &Path, version: &str, shared: Option<&Path>) {
    let bin = top.join("bin");
    let npm_bin = top.join("lib/node_modules/npm/bin");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&npm_bin).unwrap();
    fs::create_dir_all(top.join("include/node")).unwrap();
    fs::create_dir_all(top.join("share/man/man1")).unwrap();

    let log = r#"[ -z "$NVMIMIC_LAUNCH_LOG" ] || printf '%s\n' "$0" >> "$NVMIMIC_LAUNCH_LOG""#;
    let [node, npm_cli] = SCRIPTS;
    match shared {
        Some(shared) => {
            for script in [node, npm_cli] {
                std::os::unix::fs::symlink(shared.join(script), top.join(script)).unwrap();
            }
        }
        None => {
            write_script(
                &top.join(node),
                &format!(
                    "#!/bin/sh\n{log}\nif [ \"$1\" = --version ] || [ \"$1\" = -v ]; then echo {version}; else echo \"node {version}: $*\"; fi\n"
                ),
            );
            write_script(
                &top.join(npm_cli),
                &format!(
                    "#!/bin/sh\n{log}\nif [ \"$1\" = --version ]; then echo {NPM_VERSION}; else echo \"npm: $*\"; fi\n"
                ),
            );
        }
    }
    fs::write(
        top.join("lib/node_modules/npm/package.json"),
        format!(
            "{{\"name\": \"npm\", \"version\": \"{NPM_VERSION}\", \"engines\": {{\"node\": \">=0.10.0\"}}}}\n"
        ),
    )
    .unwrap();
    let (major, minor, patch) = split_version(version);
    fs::write(
        top.join("include/node/node_version.h"),
        format!(
            "#define NODE_MAJOR_VERSION {major}\n#define NODE_MINOR_VERSION {minor}\n#define NODE_PATCH_VERSION {patch}\n#define NODE_VERSION_IS_RELEASE 1\n"
        ),
    )
    .unwrap();
    fs::write(top.join("include/node/node.h"), "").unwrap();
    fs::write(top.join("share/man/man1/node.1"), "").unwrap();

    let npm = bin.join("npm");
    if fs::symlink_metadata(&npm).is_err() {
        std::os::unix::fs::symlink("../lib/node_modules/npm/bin/npm-cli.js", npm).unwrap();
    }
}

fn split_version(version: &str) -> (&str, &str, &str) {
    let mut parts = version.trim_start_matches('v').splitn(3, '.');
    (
        parts.next().unwrap(),
        parts.next().unwrap(),
        parts.next().unwrap(),
    )
}

fn write_script(path: &Path, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

pub fn sha256(path: &Path) -> String {
    let bytes = fs::read(path).unwrap();
    Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
