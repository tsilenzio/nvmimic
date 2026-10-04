# id: INS-51
# An install whose output goes to a pipe nobody reads stops at its first line.
# zsh is left out: there nvm carries on and each failed print changes what it does next, which nvmimic does not follow.
# shells: bash sh dash
# covers: nvm nvm:deactivate nvm:install nvm:ls nvm_acquire_install_lock nvm_add_iojs_prefix
# covers: nvm_alias nvm_alias_path nvm_binary_available nvm_cache_dir nvm_curl_libz_support
# covers: nvm_curl_use_compression nvm_curl_version nvm_download nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_colors nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_colors
# covers: nvm_has_executable nvm_has_system_iojs nvm_has_system_node nvm_install_binary
# covers: nvm_install_lock_name nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed
# covers: nvm_is_zsh nvm_ls nvm_ls_current nvm_ls_remote nvm_ls_remote_index_tab nvm_ls_remote_iojs
# covers: nvm_make_alias nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_print_color_code nvm_print_versions nvm_remote_version nvm_remote_versions
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_validate_implicit_alias nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
{ sleep 1; nvm install 24.15.0; } | no-such-reader 2>/dev/null; echo "rc=$?"
ls "$NVM_DIR/.cache/locks" 2>/dev/null; echo "rc=$?"
nvm ls 24.15.0; echo "rc=$?"
