# id: LST-12a
# ls-remote --lts and lts/* list every LTS release.
# setup: nvm install 22.20.0
# covers: nvm nvm:deactivate nvm:ls-remote nvm_alias_path nvm_curl_libz_support
# covers: nvm_curl_use_compression nvm_curl_version nvm_download nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_colors nvm_get_mirror nvm_grep nvm_has nvm_has_colors nvm_has_executable
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_zsh nvm_ls nvm_ls_current nvm_ls_remote nvm_ls_remote_index_tab nvm_make_alias
# covers: nvm_node_prefix nvm_normalize_lts nvm_num_version_groups nvm_print_color_code
# covers: nvm_print_versions nvm_remote_versions nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias
# covers: nvm_version_dir nvm_version_greater_than_or_equal_to
nvm ls-remote --lts; echo "rc=$?"
nvm ls-remote 'lts/*'
