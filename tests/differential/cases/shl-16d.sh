# id: SHL-16d
# A node that cannot be started counts as printing no version: current says none.
# mirror: plain
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# setup: printf '#!/nonexistent/interpreter\n' > "$NVM_DIR/versions/node/v24.15.0/bin/node"
# covers: nvm nvm:alias nvm:current nvm:deactivate nvm:install nvm:ls nvm:use nvm_add_iojs_prefix
# covers: nvm_alias nvm_alias_path nvm_change_path nvm_curl_libz_support nvm_curl_use_compression
# covers: nvm_curl_version nvm_die_on_prefix nvm_download nvm_echo nvm_ensure_default_set
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_project_dir
# covers: nvm_get_colors nvm_get_default_packages nvm_get_mirror nvm_get_os nvm_grep nvm_has
# covers: nvm_has_colors nvm_has_executable nvm_has_system_iojs nvm_has_system_node nvm_hash_reset
# covers: nvm_install_default_packages nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed
# covers: nvm_is_zsh nvm_list_aliases nvm_ls nvm_ls_current nvm_ls_remote nvm_ls_remote_index_tab
# covers: nvm_ls_remote_iojs nvm_make_alias nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_normalize_version nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_alias_file
# covers: nvm_print_alias_path nvm_print_color_code nvm_print_default_alias
# covers: nvm_print_formatted_alias nvm_print_implicit_alias nvm_print_npm_version
# covers: nvm_print_versions nvm_remote_version nvm_remote_versions nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path
# covers: nvm_validate_implicit_alias nvm_validate_install nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
nvm current; echo "rc=$?"
nvm use 24.15.0; echo "rc=$?"
nvm current; echo "rc=$?"
nvm ls; echo "rc=$?"
nvm install 24.15.0; echo "rc=$?"
nvm use 20.20.2; echo "rc=$?"
nvm current; echo "rc=$?"
