# id: UNI-09a
# uninstall removes the version directory under versions/node.
# setup: nvm install 20.20.2
# setup: nvm install 22.20.0
# covers: nvm nvm:deactivate nvm:uninstall nvm_alias nvm_alias_path nvm_cache_dir
# covers: nvm_check_file_permissions nvm_echo nvm_ensure_version_prefix nvm_err nvm_get_arch
# covers: nvm_get_download_slug nvm_get_os nvm_grep nvm_has_system_iojs nvm_has_system_node
# covers: nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version nvm_is_merged_node_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to
# covers: nvm_version_path
nvm uninstall 20 >/dev/null 2>&1
echo "rc=$?"
ls "$NVM_DIR/versions/node"
