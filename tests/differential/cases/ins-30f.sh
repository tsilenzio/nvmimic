# id: INS-30f
# With no npm anywhere on PATH, no npm version is shown.
# setup: nvm install 24.15.0
# setup: rm "$NVM_DIR/versions/node/v24.15.0/bin/npm"
# covers: nvm nvm:exec nvm:use nvm_alias nvm_alias_path nvm_auto nvm_change_path nvm_die_on_prefix
# covers: nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_find_project_dir nvm_get_os nvm_grep nvm_has nvm_hash_reset nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_ls nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_print_npm_version nvm_process_parameters nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_path
nvm use 24.15.0; echo "rc=$?"
nvm exec 24.15.0 node --version; echo "rc=$?"
