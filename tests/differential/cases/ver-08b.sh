# id: VER-08b
# system with a node outside NVM_DIR on PATH.
# setup: mkdir -p "$HOME/sys" && printf '#!/bin/sh\necho v18.0.0\n' > "$HOME/sys/node" && chmod +x "$HOME/sys/node"
# setup: PATH="$HOME/sys:$PATH"; export PATH
# covers: nvm nvm:deactivate nvm:use nvm:version nvm:which nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_grep nvm_has nvm_has_system_iojs nvm_has_system_node
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh nvm_ls nvm_match_version nvm_node_prefix
# covers: nvm_normalize_lts nvm_print_npm_version nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir
nvm version system; echo "rc=$?"
nvm which system
