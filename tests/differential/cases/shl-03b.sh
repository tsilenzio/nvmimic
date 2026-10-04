# id: SHL-03b
# Without a default alias, sourcing uses the version in .nvmrc.
# setup: (nvm install 20.20.2 && nvm unalias default)
# setup: echo 20.20.2 > .nvmrc
# covers: nvm nvm:use nvm_alias nvm_alias_path nvm_auto nvm_change_path nvm_die_on_prefix nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_nvmrc
# covers: nvm_find_project_dir nvm_find_up nvm_get_os nvm_grep nvm_has nvm_hash_reset
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_ls
# covers: nvm_ls_current nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears
# covers: nvm_num_version_groups nvm_process_nvmrc nvm_process_nvmrc_content nvm_process_parameters
# covers: nvm_rc_version nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
. "$NVM_DIR/nvm.sh"
echo "rc=$?"
node --version
