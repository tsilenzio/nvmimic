# id: LST-11
# ls-remote LTS labels and padding for current, installed and remote lines.
# setup: nvm install 24.21.0
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# covers: nvm nvm:deactivate nvm:ls-remote nvm_alias_path nvm_curl_libz_support
# covers: nvm_curl_use_compression nvm_curl_version nvm_download nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_colors nvm_get_mirror nvm_grep nvm_has nvm_has_colors nvm_has_executable
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_zsh nvm_ls nvm_ls_current nvm_ls_remote nvm_ls_remote_index_tab nvm_ls_remote_iojs
# covers: nvm_make_alias nvm_node_prefix nvm_num_version_groups nvm_print_color_code
# covers: nvm_print_versions nvm_remote_versions nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias
# covers: nvm_version_dir nvm_version_greater_than_or_equal_to
nvm ls-remote 24; echo "rc=$?"
nvm ls-remote 20; echo "rc=$?"
nvm ls-remote 26
