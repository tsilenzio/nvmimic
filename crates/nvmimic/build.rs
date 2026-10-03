use std::{env, fs, path::Path};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let pin_path = Path::new(&manifest_dir).join("../../nvm-pin.toml");
    println!("cargo::rerun-if-changed={}", pin_path.display());

    let pin = fs::read_to_string(&pin_path).expect("read nvm-pin.toml");
    for key in ["version", "tag", "commit"] {
        let value = pin
            .lines()
            .filter_map(|line| line.split_once('='))
            .find(|(k, _)| k.trim() == key)
            .map(|(_, v)| v.trim().trim_matches('"'))
            .unwrap_or_else(|| panic!("nvm-pin.toml has no `{key}`"));
        println!("cargo::rustc-env=NVM_PIN_{}={value}", key.to_uppercase());
    }
}
