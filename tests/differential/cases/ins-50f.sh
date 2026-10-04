# id: INS-50f
# With no make on PATH a source build stops after configure, and the shell says make is missing.
# setup: mkdir "$HOME/tools"
# setup: ln -s /bin/* /usr/bin/* /usr/sbin/sysctl "$HOME/tools/"
# setup: rm "$HOME/tools/make"
# covers: nvm nvm:install nvm_acquire_install_lock nvm_alias_path nvm_cache_dir nvm_cd
# covers: nvm_clang_version nvm_compare_checksum nvm_compute_checksum nvm_curl_libz_support
# covers: nvm_curl_use_compression nvm_curl_version nvm_download nvm_download_artifact nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_extract_tarball nvm_get_arch
# covers: nvm_get_artifact_compression nvm_get_checksum nvm_get_checksum_alg nvm_get_checksum_binary
# covers: nvm_get_download_slug nvm_get_make_jobs nvm_get_mirror nvm_get_os nvm_grep nvm_has
# covers: nvm_has_executable nvm_has_non_aliased nvm_install_lock_name nvm_install_source
# covers: nvm_iojs_prefix nvm_is_alias nvm_is_iojs_version nvm_is_merged_node_version
# covers: nvm_is_natural_num nvm_is_version_installed nvm_is_zsh nvm_ls_remote
# covers: nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias nvm_node_prefix
# covers: nvm_release_install_lock nvm_remote_version nvm_remote_versions nvm_strip_iojs_prefix
# covers: nvm_supports_xz nvm_validate_implicit_alias nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
PATH="$HOME/tools"
nvm install -s 24.15.0; echo "rc=$?"
ls "$NVM_DIR/.cache/src/node-v24.15.0"; echo "rc=$?"
