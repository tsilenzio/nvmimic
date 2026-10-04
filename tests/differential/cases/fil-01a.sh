# id: FIL-01a
# nvm-exec selects NODE_VERSION.
# setup: (nvm install 24.15.0)
# setup: (nvm install 20.20.2)
# covers: nvm nvm:use nvm_alias nvm_alias_path nvm_auto nvm_change_path nvm_die_on_prefix nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_project_dir
# covers: nvm_get_os nvm_grep nvm_has nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_match_version nvm_node_prefix
# covers: nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_npm_version
# covers: nvm_process_parameters nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
NODE_VERSION=20.20.2 "$NVM_DIR/nvm-exec" node --version
echo "rc=$?"
