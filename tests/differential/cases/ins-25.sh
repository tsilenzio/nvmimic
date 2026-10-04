# id: INS-25
# covers: nvm nvm:deactivate nvm:install nvm:ls nvm_acquire_install_lock nvm_add_iojs_prefix
# covers: nvm_alias nvm_alias_path nvm_cache_dir nvm_clang_version nvm_curl_libz_support
# covers: nvm_curl_use_compression nvm_curl_version nvm_download nvm_download_artifact nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_get_arch nvm_get_artifact_compression
# covers: nvm_get_checksum nvm_get_checksum_alg nvm_get_checksum_binary nvm_get_colors
# covers: nvm_get_download_slug nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_colors
# covers: nvm_has_executable nvm_has_non_aliased nvm_has_system_iojs nvm_has_system_node
# covers: nvm_install_lock_name nvm_install_source nvm_iojs_prefix nvm_is_alias nvm_is_iojs_version
# covers: nvm_is_merged_node_version nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current
# covers: nvm_ls_remote nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_print_color_code nvm_print_versions
# covers: nvm_release_install_lock nvm_remote_version nvm_remote_versions nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_supports_xz
# covers: nvm_validate_implicit_alias nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
# -s skips the binary and builds from source, which fails without a source tarball.
export NVM_MAKE_JOBS=1
nvm install -s --no-progress 24.15.0; echo "rc=$?"
nvm ls 24.15.0; echo "rc=$?"
