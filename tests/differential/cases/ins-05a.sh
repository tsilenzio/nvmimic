# id: INS-05a
# covers: nvm nvm:deactivate nvm:install nvm_alias nvm_alias_path nvm_curl_libz_support
# covers: nvm_curl_use_compression nvm_curl_version nvm_download nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_mirror nvm_get_os nvm_grep nvm_has_executable nvm_has_system_iojs
# covers: nvm_has_system_node nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh nvm_ls nvm_ls_remote
# covers: nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_remote_version nvm_remote_versions
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater_than_or_equal_to
# Malformed --reinstall-packages-from and --copy-packages-from exit 6.
nvm install --reinstall-packages-from=20 --reinstall-packages-from=18 24.15.0; echo "rc=$?"
nvm install --reinstall-packages-from=20 --copy-packages-from=18 24.15.0; echo "rc=$?"
nvm install --copy-packages-from=20 --reinstall-packages-from=18 24.15.0; echo "rc=$?"
nvm install --reinstall-packages-from= 24.15.0; echo "rc=$?"
nvm install --copy-packages-from= 24.15.0; echo "rc=$?"
nvm install --reinstall-packages-from 20 24.15.0; echo "rc=$?"
nvm install --copy-packages-from 20 24.15.0; echo "rc=$?"
nvm install 24.15.0 --reinstall-packages-from 20; echo "rc=$?"
nvm install 24.15.0 --reinstall-packages-from=20 --copy-packages-from=18; echo "rc=$?"
nvm install 24.15.0 --copy-packages-from=; echo "rc=$?"
