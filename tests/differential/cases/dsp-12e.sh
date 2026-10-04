# id: DSP-12e
# use exits 11 when NPM_CONFIG_PREFIX points outside NVM_DIR.
# setup: nvm install 20.20.2
# setup: nvm deactivate
# covers: nvm nvm:deactivate nvm:use nvm_alias nvm_alias_path nvm_change_path nvm_die_on_prefix
# covers: nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_os
# covers: nvm_grep nvm_has nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_ls nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_print_npm_version nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_path
NPM_CONFIG_PREFIX="$HOME/elsewhere" nvm use 20.20.2
