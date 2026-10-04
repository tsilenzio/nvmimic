# id: INS-06d
# --offline installs a version that is only in the download cache.
# setup: nvm install 20.20.2
# setup: nvm install 24.15.0
# setup: nvm uninstall 20.20.2
# covers: nvm nvm:deactivate nvm:install nvm:use nvm_acquire_install_lock nvm_alias nvm_alias_path
# covers: nvm_binary_available nvm_cache_dir nvm_change_path nvm_die_on_prefix nvm_download_artifact
# covers: nvm_echo nvm_ensure_default_set nvm_ensure_version_installed nvm_ensure_version_prefix
# covers: nvm_err nvm_extract_tarball nvm_find_project_dir nvm_get_arch nvm_get_artifact_compression
# covers: nvm_get_default_packages nvm_get_download_slug nvm_get_mirror nvm_get_os nvm_grep nvm_has
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_install_binary
# covers: nvm_install_binary_extract nvm_install_default_packages nvm_install_lock_name
# covers: nvm_install_npm_if_needed nvm_iojs_prefix nvm_is_iojs_version nvm_is_merged_node_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_cached nvm_ls_current nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_offline_version nvm_print_alias_file nvm_print_npm_version nvm_release_install_lock
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_sanitize_path nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_supports_xz nvm_tree_contains_path nvm_use_if_needed
# covers: nvm_validate_implicit_alias nvm_validate_install nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
nvm install --offline 20; echo "rc=$?"
