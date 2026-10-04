# id: SHL-07c
# A lowercase npm_config_prefix outside NVM_DIR is reported while sourcing.
# setup: (nvm install 24.15.0)
# covers: nvm nvm:deactivate nvm:use nvm_alias nvm_alias_path nvm_auto nvm_change_path
# covers: nvm_die_on_prefix nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_get_os nvm_grep nvm_has nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_valid_version nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current
# covers: nvm_match_version nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_print_alias_file nvm_process_parameters nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to
# covers: nvm_version_path
npm_config_prefix=/opt/elsewhere
export npm_config_prefix
. "$NVM_DIR/nvm.sh"
echo "rc=$?"
