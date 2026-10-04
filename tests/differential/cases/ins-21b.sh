# id: INS-21b
# -b with a failed binary download aborts with exit 2.
# setup: mkdir -p "$NVM_DIR/.cache" && : > "$NVM_DIR/.cache/bin"
# covers: nvm nvm:install nvm_acquire_install_lock nvm_alias_path nvm_binary_available nvm_cache_dir
# covers: nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version nvm_download
# covers: nvm_download_artifact nvm_echo nvm_ensure_version_prefix nvm_err nvm_get_arch
# covers: nvm_get_artifact_compression nvm_get_checksum nvm_get_checksum_alg nvm_get_checksum_binary
# covers: nvm_get_download_slug nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_executable
# covers: nvm_has_non_aliased nvm_install_binary nvm_install_binary_extract nvm_install_lock_name
# covers: nvm_iojs_prefix nvm_is_alias nvm_is_iojs_version nvm_is_merged_node_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls_remote nvm_ls_remote_index_tab
# covers: nvm_ls_remote_iojs nvm_make_alias nvm_node_prefix nvm_release_install_lock
# covers: nvm_remote_version nvm_remote_versions nvm_strip_iojs_prefix nvm_supports_xz
# covers: nvm_validate_implicit_alias nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
nvm install -b --no-progress 24.15.0; echo "rc=$?"
