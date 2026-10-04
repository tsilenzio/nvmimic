# id: UNI-03
# uninstall refuses the active version with the inferred-from message.
# setup: nvm install 20.20.2
# covers: nvm nvm:deactivate nvm:uninstall nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_get_os nvm_grep nvm_has_system_iojs
# covers: nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_path
nvm uninstall 20
echo "rc=$?"
nvm uninstall v20.20.2
