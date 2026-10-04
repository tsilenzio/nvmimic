# id: INS-06e
# --offline LTS request resolves through local LTS aliases.
# setup: nvm install 24.21.0
# setup: nvm install 22.20.0
# covers: nvm nvm:install nvm:use nvm_alias nvm_alias_path nvm_change_path nvm_die_on_prefix
# covers: nvm_echo nvm_ensure_default_set nvm_ensure_version_installed nvm_ensure_version_prefix
# covers: nvm_err nvm_find_project_dir nvm_get_default_packages nvm_get_os nvm_grep nvm_has
# covers: nvm_hash_reset nvm_install_default_packages nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_match_version nvm_node_prefix
# covers: nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_offline_version
# covers: nvm_print_alias_file nvm_print_npm_version nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_validate_implicit_alias nvm_validate_install nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
nvm install --offline --lts; echo "rc=$?"
nvm install --offline lts/jod; echo "rc=$?"
