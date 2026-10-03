//! Putting nvmimic's shell files into `$NVM_DIR`.

/// How `shell/nvm.sh` names the binary before it is installed.
pub const BINARY_PLACEHOLDER: &str = "\"${NVM_DIR}/.nvmimic/bin/nvm\"";

/// The `nvm.sh` to install, pointing at the binary by absolute path. nvm
/// lets `$NVM_DIR` change after sourcing, and the script itself works out
/// where it lives, so the binary cannot be found from `$NVM_DIR` alone.
pub fn bind_binary(nvm_sh: &str, nvm_link: &str) -> String {
    nvm_sh.replace(BINARY_PLACEHOLDER, &crate::shell::quote(nvm_link))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_is_replaced_and_quoted() {
        let script = format!("command {BINARY_PLACEHOLDER} __fn");
        assert_eq!(
            bind_binary(&script, "/a b/.nvmimic/bin/nvm"),
            "command '/a b/.nvmimic/bin/nvm' __fn"
        );
    }
}
