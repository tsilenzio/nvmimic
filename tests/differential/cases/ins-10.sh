# id: INS-10
# covers: nvm nvm:install nvm_alias_path nvm_curl_libz_support nvm_curl_use_compression
# covers: nvm_curl_version nvm_download nvm_echo nvm_ensure_version_prefix nvm_err nvm_get_mirror
# covers: nvm_get_os nvm_grep nvm_has_executable nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh
# covers: nvm_ls_remote nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias nvm_node_prefix
# covers: nvm_normalize_lts nvm_remote_version nvm_remote_versions nvm_strip_iojs_prefix
# covers: nvm_validate_implicit_alias nvm_version_dir nvm_version_greater_than_or_equal_to
# Version-not-found messages for plain, LTS-filtered and named LTS requests, exit 3.
nvm install 99; echo "rc=$?"
nvm install v24.99.1; echo "rc=$?"
nvm install --lts 99; echo "rc=$?"
nvm install --lts=nope; echo "rc=$?"
nvm install lts/nope; echo "rc=$?"
nvm install --lts=iron 99; echo "rc=$?"
