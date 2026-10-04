# id: FIL-01d
# nvm-exec with an uninstalled NODE_VERSION fails with 127.
# setup: (nvm install 24.15.0)
# covers: nvm nvm:use nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_auto nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_os nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_ls nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_num_version_groups nvm_process_parameters
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
NODE_VERSION=18.20.8 "$NVM_DIR/nvm-exec" node --version
echo "rc=$?"
