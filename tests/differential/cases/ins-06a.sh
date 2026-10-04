# id: INS-06a
# --offline resolves a partial version from installed versions.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# covers: nvm nvm:deactivate nvm:install nvm:use nvm_alias nvm_alias_path nvm_change_path
# covers: nvm_die_on_prefix nvm_echo nvm_ensure_default_set nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_find_project_dir nvm_get_default_packages nvm_get_os
# covers: nvm_grep nvm_has nvm_has_system_iojs nvm_has_system_node nvm_hash_reset
# covers: nvm_install_default_packages nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed
# covers: nvm_is_zsh nvm_ls nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_offline_version nvm_print_alias_file
# covers: nvm_print_npm_version nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_validate_implicit_alias nvm_validate_install nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
nvm install --offline 24; echo "rc=$?"
