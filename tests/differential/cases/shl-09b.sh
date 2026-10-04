# id: SHL-09b
# Under set -e an unguarded failing nvm call aborts the caller with its status.
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:ls nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_get_colors nvm_get_os nvm_grep nvm_has
# covers: nvm_has_colors nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current
# covers: nvm_node_prefix nvm_normalize_lts nvm_num_version_groups nvm_print_color_code
# covers: nvm_print_versions nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias nvm_version_dir
# covers: nvm_version_greater nvm_version_path
set -e
nvm ls 99.0.0
echo "not reached"
