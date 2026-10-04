# id: INS-38
# Installed versions stay until nvm uninstall removes them.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# setup: nvm install 18.20.8
# covers: nvm nvm:deactivate nvm:install nvm:ls nvm:uninstall nvm:use nvm_acquire_install_lock
# covers: nvm_alias nvm_alias_path nvm_binary_available nvm_cache_dir nvm_change_path
# covers: nvm_check_file_permissions nvm_compare_checksum nvm_compute_checksum nvm_curl_libz_support
# covers: nvm_curl_use_compression nvm_curl_version nvm_die_on_prefix nvm_download
# covers: nvm_download_artifact nvm_echo nvm_ensure_default_set nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_extract_tarball nvm_find_project_dir nvm_get_arch
# covers: nvm_get_artifact_compression nvm_get_checksum nvm_get_checksum_alg nvm_get_checksum_binary
# covers: nvm_get_colors nvm_get_default_packages nvm_get_download_slug nvm_get_mirror nvm_get_os
# covers: nvm_grep nvm_has nvm_has_colors nvm_has_executable nvm_has_non_aliased nvm_has_system_iojs
# covers: nvm_has_system_node nvm_hash_reset nvm_install_binary nvm_install_binary_extract
# covers: nvm_install_default_packages nvm_install_lock_name nvm_install_npm_if_needed
# covers: nvm_iojs_prefix nvm_is_alias nvm_is_iojs_version nvm_is_merged_node_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current nvm_ls_remote
# covers: nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_print_alias_file nvm_print_color_code nvm_print_npm_version nvm_print_versions
# covers: nvm_release_install_lock nvm_remote_version nvm_remote_versions nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_supports_xz
# covers: nvm_tree_contains_path nvm_use_if_needed nvm_validate_implicit_alias nvm_validate_install
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to
# covers: nvm_version_path
nvm install 22.20.0 >/dev/null 2>&1; echo "rc=$?"
nvm uninstall 18.20.8; echo "rc=$?"
nvm ls --no-colors --no-alias
