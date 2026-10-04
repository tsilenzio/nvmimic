# id: UNI-08
# uninstall success prints Uninstalled node <v> and exits 0.
# setup: nvm install 20.20.2
# setup: nvm install 22.20.0
# covers: nvm nvm:uninstall nvm_alias nvm_alias_path nvm_cache_dir nvm_check_file_permissions
# covers: nvm_echo nvm_ensure_version_prefix nvm_err nvm_get_arch nvm_get_download_slug nvm_get_os
# covers: nvm_grep nvm_iojs_prefix nvm_is_iojs_version nvm_is_merged_node_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_tree_contains_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
nvm uninstall 20.20.2
