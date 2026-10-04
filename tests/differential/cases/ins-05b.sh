# id: INS-05b
# covers: nvm nvm:deactivate nvm:install nvm_add_iojs_prefix nvm_alias nvm_alias_path
# covers: nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version nvm_download nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_get_mirror nvm_get_os nvm_grep nvm_has_executable
# covers: nvm_has_system_iojs nvm_has_system_node nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_remote nvm_ls_remote_index_tab
# covers: nvm_ls_remote_iojs nvm_make_alias nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_remote_version nvm_remote_versions nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
# Reinstall source equal to the target exits 4, a source not installed exits 5.
nvm install --reinstall-packages-from=24.15.0 24.15.0; echo "rc=$?"
nvm install 24.15.0 --copy-packages-from=v24.15.0; echo "rc=$?"
nvm install --reinstall-packages-from=20 24.15.0; echo "rc=$?"
nvm install 24.15.0 --copy-packages-from=18; echo "rc=$?"
