# id: ENV-04
# NVM_RC_VERSION after use and exec from .nvmrc, in the caller and the child.
# setup: (nvm install 20.20.2)
# covers: nvm nvm:deactivate nvm:exec nvm:use nvm_alias nvm_alias_path nvm_auto nvm_change_path
# covers: nvm_die_on_prefix nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_find_nvmrc nvm_find_project_dir nvm_find_up nvm_get_os nvm_grep nvm_has
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_valid_version nvm_is_version_installed nvm_is_zsh nvm_ls nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_print_npm_version nvm_process_nvmrc nvm_process_nvmrc_content nvm_process_parameters
# covers: nvm_rc_version nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
echo 20.20.2 > .nvmrc
nvm use
echo "caller NVM_RC_VERSION=${NVM_RC_VERSION-unset}"
nvm exec sh -c 'echo "exec child NVM_RC_VERSION=${NVM_RC_VERSION-unset}"'
"$NVM_DIR/nvm-exec" sh -c 'echo "nvm-exec child NVM_RC_VERSION=${NVM_RC_VERSION-unset}"'
