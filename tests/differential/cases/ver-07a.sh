# id: VER-07a
# latest is not a keyword without a user alias.
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:version nvm:version-remote nvm:which nvm_alias nvm_alias_path
# covers: nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version nvm_download nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_mirror nvm_grep
# covers: nvm_has_executable nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_zsh nvm_ls nvm_ls_remote nvm_ls_remote_index_tab
# covers: nvm_ls_remote_iojs nvm_make_alias nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_remote_version nvm_remote_versions nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater_than_or_equal_to
nvm version latest; echo "rc=$?"
nvm version-remote latest; echo "rc=$?"
nvm which latest
