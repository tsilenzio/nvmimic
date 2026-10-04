use std::{env, fs, path::Path};

/// Lines of the pinned `nvm.sh` where the shell itself can print an error
/// that nvmimic prints too, each named by text found on that line only.
const SITES: &[(&str, &str)] = &[
    (
        "DEBUG_SHELL_VERSION",
        r#"nvm_err "shell version: '$(${SHELL} --version | command head -n 1)'""#,
    ),
    (
        "DEBUG_COMMAND",
        r#"NVM_DEBUG_OUTPUT="$(${NVM_DEBUG_COMMAND} 2>&1)""#,
    ),
    ("CD", r#"\cd "$@""#),
    ("GET_OS_UNAME", r#"NVM_UNAME="$(command uname -a)""#),
    ("RC_VERSION_FD3", r#"nvm_echo "${NVM_RC_VERSION}" >&3"#),
    (
        "BUILD_CONFIGURE",
        r#"./configure --prefix="${VERSION_PATH}" $ADDITIONAL_PARAMETERS && \"#,
    ),
    (
        "BUILD_MAKE",
        r#"$make -j "${NVM_MAKE_JOBS}" ${MAKE_CXX-} && \"#,
    ),
    (
        "BUILD_MAKE_INSTALL",
        r#"$make -j "${NVM_MAKE_JOBS}" ${MAKE_CXX-} install"#,
    ),
    (
        "LINK_ABSOLUTE",
        r#"'/'*) (nvm_cd "${LINK}" && npm link) ;;"#,
    ),
    (
        "LINK_RELATIVE",
        r#"*) (nvm_cd "$(npm root -g)/../${LINK}" && npm link)"#,
    ),
];

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = Path::new(&manifest_dir).join("../..");
    let pin_path = root.join("nvm-pin.toml");
    println!("cargo::rerun-if-changed={}", pin_path.display());

    let pin = fs::read_to_string(&pin_path).expect("read nvm-pin.toml");
    let field = |key: &str| {
        pin.lines()
            .filter_map(|line| line.split_once('='))
            .find(|(k, _)| k.trim() == key)
            .map(|(_, v)| v.trim().trim_matches('"').to_string())
            .unwrap_or_else(|| panic!("nvm-pin.toml has no `{key}`"))
    };
    for key in ["version", "tag", "commit"] {
        println!(
            "cargo::rustc-env=NVM_PIN_{}={}",
            key.to_uppercase(),
            field(key)
        );
    }

    let nvm_sh = root.join(format!("tests/reference/nvm-{}/nvm.sh", field("tag")));
    println!("cargo::rerun-if-changed={}", nvm_sh.display());
    let source = fs::read_to_string(&nvm_sh).expect("read the pinned nvm.sh");
    let lines: Vec<&str> = source.lines().collect();
    let mut generated = String::new();
    for (name, text) in SITES {
        let found: Vec<usize> = (0..lines.len())
            .filter(|&index| lines[index].trim() == *text)
            .collect();
        let [index] = found[..] else {
            panic!(
                "site {name}: {} lines of nvm.sh read `{text}`, expected one",
                found.len()
            );
        };
        // nvm's functions open at the start of a line, as `name() {`.
        let (function_index, function) = (0..index)
            .rev()
            .find_map(|at| Some((at, lines[at].strip_suffix("() {")?)))
            .filter(|(_, function)| !function.contains(' '))
            .unwrap_or_else(|| panic!("site {name}: no function opens above line {}", index + 1));
        generated.push_str(&format!(
            "pub const {name}: Site = Site {{ line: {}, function: \"{function}\", function_line: {} }};\n",
            index + 1,
            function_index + 1
        ));
    }
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("sites.rs");
    fs::write(out, generated).expect("write sites.rs");
}
