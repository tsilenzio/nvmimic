# id: INS-36a
# PREFIX and any-case NPM_CONFIG_PREFIX outside NVM_DIR make nvm use exit 11 and deactivate.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# covers: nvm nvm:current nvm:deactivate nvm:use nvm_alias nvm_alias_path nvm_change_path
# covers: nvm_die_on_prefix nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_find_project_dir nvm_get_os nvm_grep nvm_has nvm_hash_reset nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_version_installed nvm_ls nvm_ls_current nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_print_npm_version nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
export PREFIX=/opt/elsewhere
nvm use 24.15.0; echo "rc=$?"
nvm current
unset PREFIX
export npm_config_prefix=/opt/elsewhere
nvm use 24.15.0; echo "rc=$?"
nvm current
unset npm_config_prefix
export NPM_CONFIG_PREFIX="$NVM_DIR/versions/node/v24.15.0"
nvm use 24.15.0; echo "rc=$?"
unset NPM_CONFIG_PREFIX
