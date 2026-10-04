# id: INS-12c
# Already installed with --reinstall-packages-from runs reinstall-packages.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# covers: nvm nvm:deactivate nvm:install nvm:reinstall-packages nvm:use nvm_alias nvm_alias_path
# covers: nvm_change_path nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version
# covers: nvm_die_on_prefix nvm_download nvm_echo nvm_ensure_default_set
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_project_dir
# covers: nvm_get_default_packages nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_executable
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_install_default_packages
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_ls
# covers: nvm_ls_current nvm_ls_remote nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias
# covers: nvm_match_version nvm_node_prefix nvm_normalize_lts nvm_npm_global_modules
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_alias_file nvm_print_npm_version
# covers: nvm_remote_version nvm_remote_versions nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias
# covers: nvm_validate_install nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
nvm install --reinstall-packages-from=20 24.15.0; echo "rc=$?"
