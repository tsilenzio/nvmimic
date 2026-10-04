# id: INS-06f
# --offline with a cached version but no cached archive reports the missing archive.
# setup: mkdir -p "$NVM_DIR/.cache/src/node-v20.20.2"
# covers: nvm nvm:deactivate nvm:install nvm_acquire_install_lock nvm_alias nvm_alias_path
# covers: nvm_binary_available nvm_cache_dir nvm_clang_version nvm_download_artifact nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_get_arch nvm_get_artifact_compression
# covers: nvm_get_download_slug nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_system_iojs
# covers: nvm_has_system_node nvm_install_binary nvm_install_binary_extract nvm_install_lock_name
# covers: nvm_install_source nvm_iojs_prefix nvm_is_iojs_version nvm_is_merged_node_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_cached nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_offline_version nvm_release_install_lock nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_supports_xz
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
export NVM_MAKE_JOBS=1
nvm install --offline -b 20; echo "rc=$?"
nvm install --offline 20; echo "rc=$?"
