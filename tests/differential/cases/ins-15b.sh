# id: INS-15b
# --alias and --default on the already-installed path, exit code propagated.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# covers: nvm nvm:alias nvm:deactivate nvm:install nvm:use nvm_alias nvm_alias_path nvm_change_path
# covers: nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version nvm_die_on_prefix
# covers: nvm_download nvm_echo nvm_ensure_default_set nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_find_project_dir nvm_get_colors
# covers: nvm_get_default_packages nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_colors
# covers: nvm_has_executable nvm_has_system_iojs nvm_has_system_node nvm_hash_reset
# covers: nvm_install_default_packages nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed
# covers: nvm_is_zsh nvm_list_aliases nvm_ls nvm_ls_current nvm_ls_remote nvm_ls_remote_index_tab
# covers: nvm_ls_remote_iojs nvm_make_alias nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_alias_file nvm_print_alias_path
# covers: nvm_print_color_code nvm_print_formatted_alias nvm_print_npm_version nvm_remote_version
# covers: nvm_remote_versions nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias nvm_validate_install
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to
# covers: nvm_version_path
nvm install --alias=dev 24.15.0; echo "rc=$?"
nvm install --default 20; echo "rc=$?"
nvm install --alias=node 24.15.0; echo "rc=$?"
nvm install --alias=a#b 24.15.0; echo "rc=$?"
nvm install --alias=x/y 24.15.0; echo "rc=$?"
nvm alias default
