# id: INS-19
# A fresh install switches to the new version.
# setup: nvm install 24.15.0
# covers: nvm nvm:current nvm:install nvm:use nvm_acquire_install_lock nvm_alias nvm_alias_path
# covers: nvm_binary_available nvm_cache_dir nvm_change_path nvm_compare_checksum
# covers: nvm_compute_checksum nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version
# covers: nvm_die_on_prefix nvm_download nvm_download_artifact nvm_echo nvm_ensure_default_set
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_extract_tarball
# covers: nvm_find_project_dir nvm_get_arch nvm_get_artifact_compression nvm_get_checksum
# covers: nvm_get_checksum_alg nvm_get_checksum_binary nvm_get_default_packages
# covers: nvm_get_download_slug nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_executable
# covers: nvm_has_non_aliased nvm_hash_reset nvm_install_binary nvm_install_binary_extract
# covers: nvm_install_default_packages nvm_install_lock_name nvm_install_npm_if_needed
# covers: nvm_iojs_prefix nvm_is_alias nvm_is_iojs_version nvm_is_merged_node_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current nvm_ls_remote
# covers: nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_print_alias_file nvm_print_npm_version nvm_release_install_lock nvm_remote_version
# covers: nvm_remote_versions nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_supports_xz nvm_tree_contains_path nvm_use_if_needed nvm_validate_implicit_alias
# covers: nvm_validate_install nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
nvm install 20.20.2 2>"$HOME/err"; echo "rc=$?"
grep -v -E '^(Downloading [a-z]+://|Computing checksum|Checksums match|Local cache found)|%$' "$HOME/err" >&2 || :
nvm current
node --version
