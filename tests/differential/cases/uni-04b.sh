# id: UNI-04b
# The inferred-from form when the pattern resolves to an invalid install.
# setup: mkdir -p "$NVM_DIR/versions/node/v18.20.8"
# covers: nvm nvm:deactivate nvm:uninstall nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_get_os nvm_grep nvm_has_system_iojs
# covers: nvm_has_system_node nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed
# covers: nvm_is_zsh nvm_ls nvm_ls_current nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
nvm uninstall 18
