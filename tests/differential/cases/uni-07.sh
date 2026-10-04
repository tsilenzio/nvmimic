# id: UNI-07
# uninstall deletes aliases whose file contains the full version.
# setup: nvm install 20.20.2
# setup: nvm install 22.20.0
# setup: nvm alias exact v20.20.2
# setup: nvm alias major 20
# covers: nvm nvm:alias nvm:deactivate nvm:unalias nvm:uninstall nvm_add_iojs_prefix nvm_alias
# covers: nvm_alias_path nvm_cache_dir nvm_check_file_permissions nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_arch nvm_get_colors nvm_get_download_slug nvm_get_os nvm_grep nvm_has
# covers: nvm_has_colors nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_merged_node_version nvm_is_version_installed nvm_is_zsh
# covers: nvm_list_aliases nvm_ls nvm_ls_current nvm_node_prefix nvm_normalize_lts
# covers: nvm_normalize_version nvm_num_version_groups nvm_print_alias_file nvm_print_alias_path
# covers: nvm_print_color_code nvm_print_default_alias nvm_print_formatted_alias
# covers: nvm_print_implicit_alias nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
nvm uninstall 20.20.2
echo "rc=$?"
nvm alias
