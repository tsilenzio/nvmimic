# id: FIL-01f
# nvm-exec with an .nvmrc naming an uninstalled version reports it and fails with 127.
# setup: (nvm install 24.15.0)
# covers: nvm nvm:use nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_auto nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_nvmrc nvm_find_up
# covers: nvm_get_os nvm_grep nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed
# covers: nvm_is_zsh nvm_ls nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_process_nvmrc nvm_process_nvmrc_content nvm_process_parameters nvm_rc_version
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
echo 18.20.8 > .nvmrc
"$NVM_DIR/nvm-exec" node --version
echo "rc=$?"
