# id: DSP-06a
# covers: nvm nvm:install nvm_alias_path nvm_curl_libz_support nvm_curl_use_compression
# covers: nvm_curl_version nvm_download nvm_echo nvm_ensure_version_prefix nvm_err nvm_get_mirror
# covers: nvm_get_os nvm_grep nvm_has_executable nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh
# covers: nvm_ls_remote nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias nvm_node_prefix
# covers: nvm_remote_version nvm_remote_versions nvm_strip_iojs_prefix nvm_validate_implicit_alias
# covers: nvm_version_dir nvm_version_greater_than_or_equal_to
# install treats --ltsfoo as the version, not as an LTS flag.
nvm install --ltsfoo
