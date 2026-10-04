# id: SHL-15a
# NVM_SYMLINK_CURRENT=true points $NVM_DIR/current at the version in use.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# covers: nvm nvm:use nvm_alias nvm_alias_path nvm_change_path nvm_die_on_prefix nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_project_dir
# covers: nvm_get_os nvm_grep nvm_has nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_ls nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_npm_version nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
NVM_SYMLINK_CURRENT=true
export NVM_SYMLINK_CURRENT
nvm use 24.15.0 >/dev/null
readlink "$NVM_DIR/current"
nvm use 20.20.2 >/dev/null
readlink "$NVM_DIR/current"
"$NVM_DIR/current/bin/node" --version
