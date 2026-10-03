//! A local stand-in for nodejs.org/dist that both real nvm and nvmimic
//! install from. Each release is a tiny tree whose `node` and `npm` are shell
//! scripts, packed the way the real dist directory packs them.

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
        .collect()
}

pub struct Mirror {
    pub url: String,
    pub dist: PathBuf,
    pub requests: Arc<Mutex<Vec<String>>>,
}

static SHARED: OnceLock<Mirror> = OnceLock::new();

/// One mirror per test process, since building every release archive is the
/// slow part. Its directory is left behind on exit and swept by later runs.
pub fn shared() -> &'static Mirror {
    SHARED.get_or_init(|| {
        let parent = std::env::temp_dir();
        sweep_stale(&parent);
        let root = parent.join(format!("nvmimic-mirror-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        Mirror::start(&root)
    })
}

fn sweep_stale(parent: &Path) {
    let Ok(entries) = fs::read_dir(parent) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let old = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|time| time.elapsed().ok())
            .is_some_and(|age| age.as_secs() > 3600);
        if name.starts_with("nvmimic-mirror-") && old {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}

impl Mirror {
    pub fn start(root: &Path) -> Mirror {
        let dist = root.join("dist");
        build_dist(&dist, root);
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
            dist,
            requests,
        }
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

    let file = dist.join(path);
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

const INDEX_HEADER: &str =
    "version\tdate\tfiles\tnpm\tv8\tuv\tzlib\topenssl\tmodules\tlts\tsecurity\n";

fn build_dist(dist: &Path, scratch: &Path) {
    fs::create_dir_all(dist).unwrap();
    let mut tab = String::from(INDEX_HEADER);

    for release in RELEASES {
        let version = format!("v{}", release.version);
        let dir = dist.join(&version);
        fs::create_dir_all(&dir).unwrap();

        let mut sums = String::new();
        for platform in platforms(release.version) {
            let name = format!("node-{version}-{platform}");
            let tree = scratch.join("trees").join(&name);
            build_tree(&tree.join(&name), &version);

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

        let lts = release.lts.unwrap_or("-");
        tab.push_str(&format!(
            "{version}\t{}\t{}\t{NPM_VERSION}\t-\t-\t-\t-\t-\t{lts}\t-\n",
            release.date,
            files(release.version).join(",")
        ));
    }

    fs::write(dist.join("index.tab"), tab).unwrap();

    // nvm also lists io.js releases. An empty index keeps it off iojs.org.
    fs::create_dir_all(dist.join("iojs")).unwrap();
    fs::write(dist.join("iojs/index.tab"), INDEX_HEADER).unwrap();
}

/// The layout nvm's post-install validation checks: an executable
/// `bin/node` and a `bin/npm` symlink that resolves. Both scripts note each
/// launch in `$NVMIMIC_LAUNCH_LOG` when the harness counts launches.
fn build_tree(top: &Path, version: &str) {
    let bin = top.join("bin");
    let npm_bin = top.join("lib/node_modules/npm/bin");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&npm_bin).unwrap();
    fs::create_dir_all(top.join("include/node")).unwrap();
    fs::create_dir_all(top.join("share/man/man1")).unwrap();

    let log = r#"[ -z "$NVMIMIC_LAUNCH_LOG" ] || printf '%s\n' "$0" >> "$NVMIMIC_LAUNCH_LOG""#;
    write_script(
        &bin.join("node"),
        &format!(
            "#!/bin/sh\n{log}\nif [ \"$1\" = --version ] || [ \"$1\" = -v ]; then echo {version}; else echo \"node {version}: $*\"; fi\n"
        ),
    );
    write_script(
        &npm_bin.join("npm-cli.js"),
        &format!(
            "#!/bin/sh\n{log}\nif [ \"$1\" = --version ]; then echo {NPM_VERSION}; else echo \"npm: $*\"; fi\n"
        ),
    );
    fs::write(
        top.join("lib/node_modules/npm/package.json"),
        format!("{{\"name\": \"npm\", \"version\": \"{NPM_VERSION}\"}}\n"),
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
