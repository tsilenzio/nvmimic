# id: VER-12a
# lts/-N counts back through the lts alias files.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
# covers: nvm nvm:deactivate nvm:ls-remote nvm:version nvm:which nvm_alias nvm_alias_path
# covers: nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version nvm_download nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_colors
# covers: nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_colors nvm_has_executable
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current nvm_ls_remote
# covers: nvm_ls_remote_index_tab nvm_make_alias nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_print_alias_file nvm_print_color_code nvm_print_versions
# covers: nvm_remote_versions nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
nvm version lts/-1; echo "rc=$?"
nvm version lts/-2; echo "rc=$?"
nvm which lts/-1; echo "rc=$?"
nvm ls-remote lts/-1
