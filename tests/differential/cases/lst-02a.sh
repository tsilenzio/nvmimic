# id: LST-02a
# ls appends a system line when a system node exists.
# setup: mkdir -p "$HOME/sys" && printf '#!/bin/sh\necho v18.0.0\n' > "$HOME/sys/node" && chmod +x "$HOME/sys/node"
# setup: PATH="$HOME/sys:$PATH"; export PATH
# setup: nvm install 22.20.0
# covers: nvm nvm:deactivate nvm:ls nvm_echo nvm_ensure_version_prefix nvm_get_colors nvm_grep
# covers: nvm_has nvm_has_colors nvm_has_system_iojs nvm_has_system_node nvm_hash_reset
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh nvm_ls nvm_ls_current nvm_node_prefix
# covers: nvm_num_version_groups nvm_print_color_code nvm_print_versions nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path
# covers: nvm_version_dir
nvm ls --no-alias
