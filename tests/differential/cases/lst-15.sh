# id: LST-15
# Listings carry no color codes when stdout is not a TTY.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
# covers: nvm nvm:alias nvm:deactivate nvm:ls nvm:ls-remote nvm_add_iojs_prefix nvm_alias
# covers: nvm_alias_path nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version
# covers: nvm_download nvm_echo nvm_ensure_version_prefix nvm_err nvm_get_colors nvm_get_mirror
# covers: nvm_get_os nvm_grep nvm_has nvm_has_colors nvm_has_executable nvm_has_system_iojs
# covers: nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_list_aliases nvm_ls nvm_ls_current nvm_ls_remote
# covers: nvm_ls_remote_index_tab nvm_make_alias nvm_node_prefix nvm_normalize_lts
# covers: nvm_normalize_version nvm_num_version_groups nvm_print_alias_file nvm_print_alias_path
# covers: nvm_print_color_code nvm_print_default_alias nvm_print_formatted_alias
# covers: nvm_print_implicit_alias nvm_print_versions nvm_remote_versions nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
nvm ls; echo "rc=$?"
nvm ls-remote --lts; echo "rc=$?"
nvm alias
