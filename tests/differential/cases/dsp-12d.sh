# id: DSP-12d
# use on a version directory that fails install validation.
# setup: mkdir -p "$NVM_DIR/versions/node/v18.20.8"
# covers: nvm nvm:deactivate nvm:use nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_os nvm_grep
# covers: nvm_has_system_iojs nvm_has_system_node nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_match_version nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
nvm use 18
