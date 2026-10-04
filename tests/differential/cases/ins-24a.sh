# id: INS-24a
# covers: nvm nvm:alias nvm:install nvm:install-latest-npm nvm:use nvm_acquire_install_lock
# covers: nvm_alias nvm_alias_path nvm_binary_available nvm_cache_dir nvm_change_path
# covers: nvm_compare_checksum nvm_compute_checksum nvm_curl_libz_support nvm_curl_use_compression
# covers: nvm_curl_version nvm_die_on_prefix nvm_download nvm_download_artifact nvm_echo
# covers: nvm_ensure_default_set nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_extract_tarball nvm_find_project_dir nvm_get_arch nvm_get_artifact_compression
# covers: nvm_get_checksum nvm_get_checksum_alg nvm_get_checksum_binary nvm_get_colors
# covers: nvm_get_default_packages nvm_get_download_slug nvm_get_mirror nvm_get_os nvm_grep nvm_has
# covers: nvm_has_colors nvm_has_executable nvm_has_non_aliased nvm_hash_reset nvm_install_binary
# covers: nvm_install_binary_extract nvm_install_default_packages nvm_install_latest_npm
# covers: nvm_install_lock_name nvm_install_npm_if_needed nvm_iojs_prefix nvm_is_alias
# covers: nvm_is_iojs_version nvm_is_merged_node_version nvm_is_version_installed nvm_is_zsh nvm_ls
# covers: nvm_ls_current nvm_ls_remote nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias
# covers: nvm_match_version nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears
# covers: nvm_num_version_groups nvm_print_color_code nvm_print_formatted_alias
# covers: nvm_print_npm_version nvm_release_install_lock nvm_remote_version nvm_remote_versions
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_supports_xz
# covers: nvm_tree_contains_path nvm_use_if_needed nvm_validate_implicit_alias nvm_validate_install
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to
# covers: nvm_version_path
# --latest-npm on a fresh install runs install-latest-npm.
nvm install --latest-npm 24.15.0 2>"$HOME/err"; echo "rc=$?"
grep -v -E '^(Downloading [a-z]+://|Computing checksum|Checksums match|Local cache found)|%$' "$HOME/err" >&2 || :
