# id: SHL-07d
# use with a PREFIX conflict fails with 11 and --delete-prefix does not clear PREFIX.
# setup: (nvm install 24.15.0)
# covers: nvm nvm:deactivate nvm:use nvm_alias nvm_alias_path nvm_change_path nvm_die_on_prefix
# covers: nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_os
# covers: nvm_grep nvm_has nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_ls nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_print_npm_version nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
PREFIX=/opt/elsewhere
export PREFIX
nvm use 24.15.0
echo "rc=$?"
nvm use --delete-prefix 24.15.0
echo "rc=$?"
