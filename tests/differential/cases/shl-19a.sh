# id: SHL-19a
# exec runs the command through nvm-exec with NODE_VERSION, PATH and the nvm variables set.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# covers: nvm nvm:exec nvm:use nvm_alias nvm_alias_path nvm_auto nvm_change_path nvm_die_on_prefix
# covers: nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_find_project_dir nvm_get_os nvm_grep nvm_has nvm_hash_reset nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_ls nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_print_npm_version nvm_process_parameters nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_path
nvm exec 20.20.2 sh -c 'echo "NODE_VERSION=$NODE_VERSION"; node --version; echo "$NVM_BIN"; echo "$NVM_INC"; echo "MANPATH=$MANPATH"; echo "NVM_CD_FLAGS=[$NVM_CD_FLAGS]"'
echo "rc=$?"
node --version
