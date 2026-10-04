# id: SHL-19b
# exec fails when the prefix checks inside nvm-exec fail.
# setup: (nvm install 20.20.2)
# covers: nvm nvm:deactivate nvm:exec nvm:use nvm_alias nvm_alias_path nvm_auto nvm_change_path
# covers: nvm_die_on_prefix nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_get_os nvm_grep nvm_has nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_match_version nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_print_npm_version nvm_process_parameters
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
PREFIX=/opt/elsewhere
export PREFIX
nvm exec 20.20.2 node --version
echo "rc=$?"
