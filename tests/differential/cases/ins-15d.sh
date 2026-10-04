# id: INS-15d
# --lts with --default on the already-installed path.
# setup: nvm install 24.21.0
# covers: nvm nvm:alias nvm:install nvm:unalias nvm:use nvm_alias nvm_alias_path nvm_change_path
# covers: nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version nvm_die_on_prefix
# covers: nvm_download nvm_echo nvm_ensure_default_set nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_find_project_dir nvm_get_default_packages
# covers: nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_colors nvm_has_executable
# covers: nvm_hash_reset nvm_install_default_packages nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_list_aliases nvm_ls nvm_ls_current nvm_ls_remote
# covers: nvm_ls_remote_index_tab nvm_make_alias nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_alias_file nvm_print_alias_path
# covers: nvm_print_npm_version nvm_remote_version nvm_remote_versions nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_tree_contains_path
# covers: nvm_validate_implicit_alias nvm_validate_install nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
nvm install --lts --default; echo "rc=$?"
nvm alias default; echo "rc=$?"
