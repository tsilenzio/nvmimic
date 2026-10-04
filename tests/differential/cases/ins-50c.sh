# id: INS-50c
# covers: nvm nvm:alias nvm:deactivate nvm:install nvm:ls nvm_acquire_install_lock
# covers: nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_cache_dir nvm_cd nvm_clang_version
# covers: nvm_compare_checksum nvm_compute_checksum nvm_curl_libz_support nvm_curl_use_compression
# covers: nvm_curl_version nvm_download nvm_download_artifact nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_extract_tarball nvm_get_arch nvm_get_artifact_compression nvm_get_checksum
# covers: nvm_get_checksum_alg nvm_get_checksum_binary nvm_get_colors nvm_get_download_slug
# covers: nvm_get_make_jobs nvm_get_mirror nvm_get_os nvm_grep nvm_has nvm_has_colors
# covers: nvm_has_executable nvm_has_non_aliased nvm_has_system_iojs nvm_has_system_node
# covers: nvm_install_lock_name nvm_install_source nvm_iojs_prefix nvm_is_alias nvm_is_iojs_version
# covers: nvm_is_merged_node_version nvm_is_natural_num nvm_is_version_installed nvm_is_zsh
# covers: nvm_list_aliases nvm_ls nvm_ls_current nvm_ls_remote nvm_ls_remote_index_tab
# covers: nvm_ls_remote_iojs nvm_make_alias nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_print_alias_file nvm_print_alias_path nvm_print_color_code nvm_print_default_alias
# covers: nvm_print_formatted_alias nvm_print_implicit_alias nvm_print_versions
# covers: nvm_release_install_lock nvm_remote_version nvm_remote_versions nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_supports_xz
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
# A failing configure fails the install, removes the unpacked source and leaves nothing installed.
NVMIMIC_TEST_BUILD_FAILS=configure; export NVMIMIC_TEST_BUILD_FAILS
nvm install -s 24.15.0; echo "rc=$?"
nvm ls; echo "rc=$?"
ls "$NVM_DIR/.cache/src/node-v24.15.0"; echo "rc=$?"
