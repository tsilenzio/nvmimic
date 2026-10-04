# id: FIL-06b
# covers: nvm nvm:deactivate nvm:ls-remote nvm_alias_path nvm_curl_libz_support
# covers: nvm_curl_use_compression nvm_curl_version nvm_download nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_colors nvm_get_mirror nvm_grep nvm_has nvm_has_colors nvm_has_executable
# covers: nvm_has_system_iojs nvm_has_system_node nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh
# covers: nvm_ls nvm_ls_current nvm_ls_remote nvm_ls_remote_index_tab nvm_make_alias nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_print_color_code nvm_print_versions
# covers: nvm_remote_versions nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_validate_implicit_alias nvm_version_dir
# covers: nvm_version_greater_than_or_equal_to
# A remote listing rewrites stale LTS alias files.
mkdir -p "$NVM_DIR/alias/lts"
echo v0.0.1 > "$NVM_DIR/alias/lts/iron"
echo lts/bogus > "$NVM_DIR/alias/lts/*"
nvm ls-remote --lts >/dev/null
cat "$NVM_DIR/alias/lts/iron"
cat "$NVM_DIR/alias/lts/*"
