# id: DLD-01a
# covers: nvm nvm:deactivate nvm:ls-remote nvm_alias_path nvm_curl_libz_support
# covers: nvm_curl_use_compression nvm_curl_version nvm_download nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_colors nvm_get_mirror nvm_grep nvm_has nvm_has_colors nvm_has_executable
# covers: nvm_has_system_iojs nvm_has_system_node nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh
# covers: nvm_ls nvm_ls_current nvm_ls_remote nvm_ls_remote_index_tab nvm_ls_remote_iojs
# covers: nvm_node_prefix nvm_num_version_groups nvm_print_color_code nvm_print_versions
# covers: nvm_remote_versions nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_validate_implicit_alias nvm_version_dir
# covers: nvm_version_greater_than_or_equal_to
# Invalid NVM_NODEJS_ORG_MIRROR values are rejected with the URL-only message.
for m in 'https://example.com/a b' "https://example.com/'x" 'https://example.com/$(id)' 'ftp://example.com/dist' 'https://example.com/dist?x=1' 'example.com/dist'; do
  NVM_NODEJS_ORG_MIRROR="$m" nvm ls-remote 24.15.0; echo "rc=$?"
done
