# id: INS-27
# covers: nvm nvm:install nvm_acquire_install_lock nvm_alias_path nvm_cache_dir nvm_clang_version
# covers: nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version nvm_download
# covers: nvm_download_artifact nvm_echo nvm_ensure_version_prefix nvm_err nvm_get_arch
# covers: nvm_get_artifact_compression nvm_get_checksum nvm_get_checksum_alg nvm_get_checksum_binary
# covers: nvm_get_download_slug nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_executable
# covers: nvm_has_non_aliased nvm_install_lock_name nvm_install_source nvm_iojs_prefix nvm_is_alias
# covers: nvm_is_iojs_version nvm_is_merged_node_version nvm_is_version_installed nvm_is_zsh
# covers: nvm_ls_remote nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias nvm_node_prefix
# covers: nvm_release_install_lock nvm_remote_version nvm_remote_versions nvm_strip_iojs_prefix
# covers: nvm_supports_xz nvm_validate_implicit_alias nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
# Trailing arguments on a source build print the additional options line.
export NVM_MAKE_JOBS=1
nvm install -s --no-progress 24.15.0 --with-intl=none --without-npm; echo "rc=$?"
