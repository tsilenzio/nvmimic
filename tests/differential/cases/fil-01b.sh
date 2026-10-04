# id: FIL-01b
# nvm-exec without NODE_VERSION uses .nvmrc.
# setup: (nvm install 24.15.0)
# setup: (nvm install 20.20.2)
# covers: nvm nvm:use nvm_alias nvm_alias_path nvm_auto nvm_change_path nvm_die_on_prefix nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_nvmrc
# covers: nvm_find_project_dir nvm_find_up nvm_get_os nvm_grep nvm_has nvm_hash_reset
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_ls
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_print_npm_version nvm_process_nvmrc nvm_process_nvmrc_content nvm_process_parameters
# covers: nvm_rc_version nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
echo 20.20.2 > .nvmrc
"$NVM_DIR/nvm-exec" node --version
echo "rc=$?"
