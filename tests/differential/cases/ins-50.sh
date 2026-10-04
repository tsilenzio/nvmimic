# id: INS-50
# covers: nvm nvm:alias nvm:deactivate nvm:install nvm:ls nvm:use nvm_acquire_install_lock
# covers: nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_cache_dir nvm_cd nvm_change_path
# covers: nvm_clang_version nvm_compare_checksum nvm_compute_checksum nvm_curl_libz_support
# covers: nvm_curl_use_compression nvm_curl_version nvm_die_on_prefix nvm_download
# covers: nvm_download_artifact nvm_echo nvm_ensure_default_set nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_extract_tarball nvm_find_project_dir nvm_get_arch
# covers: nvm_get_artifact_compression nvm_get_checksum nvm_get_checksum_alg nvm_get_checksum_binary
# covers: nvm_get_colors nvm_get_default_packages nvm_get_download_slug nvm_get_make_jobs
# covers: nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_colors nvm_has_executable
# covers: nvm_has_non_aliased nvm_has_system_iojs nvm_has_system_node nvm_hash_reset
# covers: nvm_install_default_packages nvm_install_lock_name nvm_install_npm_if_needed
# covers: nvm_install_source nvm_iojs_prefix nvm_is_alias nvm_is_iojs_version
# covers: nvm_is_merged_node_version nvm_is_natural_num nvm_is_version_installed nvm_is_zsh
# covers: nvm_list_aliases nvm_ls nvm_ls_current nvm_ls_remote nvm_ls_remote_index_tab
# covers: nvm_ls_remote_iojs nvm_make_alias nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_normalize_version nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_alias_file
# covers: nvm_print_alias_path nvm_print_color_code nvm_print_default_alias
# covers: nvm_print_formatted_alias nvm_print_implicit_alias nvm_print_npm_version
# covers: nvm_print_versions nvm_release_install_lock nvm_remote_version nvm_remote_versions
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_supports_xz nvm_tree_contains_path nvm_use_if_needed nvm_validate_implicit_alias
# covers: nvm_validate_install nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
# install -s builds from the source archive: configure, make, make install.
nvm install -s 24.15.0; echo "rc=$?"
nvm ls; echo "rc=$?"
node --version; echo "rc=$?"
ls "$NVM_DIR/.cache/src" "$NVM_DIR/.cache/src/node-v24.15.0"; echo "rc=$?"
