# id: INS-17a
# A held install lock times out after NVM_INSTALL_LOCK_TIMEOUT seconds with exit 1.
# setup: mkdir -p "$NVM_DIR/.cache/locks/v24.15.0"
# covers: nvm nvm:install nvm_acquire_install_lock nvm_alias_path nvm_cache_dir
# covers: nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version nvm_download nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_get_mirror nvm_get_os nvm_grep nvm_has_executable
# covers: nvm_install_lock_name nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed
# covers: nvm_is_zsh nvm_ls_remote nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias
# covers: nvm_node_prefix nvm_remote_version nvm_remote_versions nvm_strip_iojs_prefix
# covers: nvm_validate_implicit_alias nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
NVM_INSTALL_LOCK_TIMEOUT=1 nvm install 24.15.0; echo "rc=$?"
