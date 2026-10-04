# id: INS-17c
# The install lock is per version, a lock on another version does not block.
# setup: mkdir -p "$NVM_DIR/.cache/locks/v20.20.2"
# covers: nvm nvm:alias nvm:install nvm:use nvm_acquire_install_lock nvm_alias nvm_alias_path
# covers: nvm_binary_available nvm_cache_dir nvm_change_path nvm_compare_checksum
# covers: nvm_compute_checksum nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version
# covers: nvm_die_on_prefix nvm_download nvm_download_artifact nvm_echo nvm_ensure_default_set
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_extract_tarball
# covers: nvm_find_project_dir nvm_get_arch nvm_get_artifact_compression nvm_get_checksum
# covers: nvm_get_checksum_alg nvm_get_checksum_binary nvm_get_colors nvm_get_default_packages
# covers: nvm_get_download_slug nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_colors
# covers: nvm_has_executable nvm_has_non_aliased nvm_hash_reset nvm_install_binary
# covers: nvm_install_binary_extract nvm_install_default_packages nvm_install_lock_name
# covers: nvm_install_npm_if_needed nvm_iojs_prefix nvm_is_alias nvm_is_iojs_version
# covers: nvm_is_merged_node_version nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current
# covers: nvm_ls_remote nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_print_color_code nvm_print_formatted_alias nvm_print_npm_version
# covers: nvm_release_install_lock nvm_remote_version nvm_remote_versions nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_supports_xz nvm_tree_contains_path
# covers: nvm_use_if_needed nvm_validate_implicit_alias nvm_validate_install nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
NVM_INSTALL_LOCK_TIMEOUT=1 nvm install 24.15.0 >/dev/null 2>&1; echo "rc=$?"
NVM_INSTALL_LOCK_TIMEOUT=0 nvm install 20.20.2; echo "rc=$?"
