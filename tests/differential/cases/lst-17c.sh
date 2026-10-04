# id: LST-17c
# current prints system for a node outside NVM_DIR.
# setup: mkdir -p "$HOME/sys" && printf '#!/bin/sh\necho v18.0.0\n' > "$HOME/sys/node" && chmod +x "$HOME/sys/node"
# setup: PATH="$HOME/sys:$PATH"; export PATH
# covers: nvm nvm:current nvm_echo nvm_ls_current nvm_tree_contains_path nvm_version nvm_version_dir
nvm current
