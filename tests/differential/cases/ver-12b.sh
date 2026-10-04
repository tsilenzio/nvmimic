# id: VER-12b
# lts/-N beyond the known LTS lines fails.
# setup: nvm install 22.20.0
# covers: nvm nvm:deactivate nvm:install nvm:ls-remote nvm:version nvm:which nvm_alias
# covers: nvm_alias_path nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version
# covers: nvm_download nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_get_colors nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_colors
# covers: nvm_has_executable nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_zsh nvm_ls nvm_ls_current nvm_ls_remote nvm_ls_remote_index_tab
# covers: nvm_make_alias nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_print_color_code nvm_print_versions nvm_remote_version nvm_remote_versions
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_tree_contains_path nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater_than_or_equal_to
nvm version lts/-5; echo "rc=$?"
nvm which lts/-9; echo "rc=$?"
nvm ls-remote lts/-9; echo "rc=$?"
nvm install lts/-9
