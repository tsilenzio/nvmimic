# id: DSP-28b
# covers: nvm nvm:version-remote nvm_alias_path nvm_curl_libz_support nvm_curl_use_compression
# covers: nvm_curl_version nvm_download nvm_echo nvm_ensure_version_prefix nvm_err nvm_get_mirror
# covers: nvm_grep nvm_has_executable nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh nvm_ls_remote
# covers: nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias nvm_node_prefix
# covers: nvm_normalize_version nvm_print_implicit_alias nvm_remote_version nvm_remote_versions
# covers: nvm_strip_iojs_prefix nvm_validate_implicit_alias nvm_version_dir
# covers: nvm_version_greater_than_or_equal_to
# version-remote default pattern, empty LTS filter and N/A results.
nvm version-remote
echo "rc=$?"
nvm version-remote --lts=
echo "rc=$?"
nvm version-remote 99
echo "rc=$?"
nvm version-remote default
echo "rc=$?"
nvm version-remote latest
echo "rc=$?"
nvm version-remote --foo
