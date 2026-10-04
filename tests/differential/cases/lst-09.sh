# id: LST-09
# Alias line shapes for collapsed, expanded and uninstalled targets.
# setup: nvm install 24.15.0
# setup: nvm alias exact v24.15.0
# setup: nvm alias novee 24.15.0
# setup: nvm alias missing 20.20.2
# covers: nvm nvm:alias nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_get_colors nvm_get_os nvm_has nvm_has_colors
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_list_aliases
# covers: nvm_ls nvm_ls_current nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_print_alias_file nvm_print_alias_path nvm_print_color_code nvm_print_formatted_alias
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_tree_contains_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
nvm alias exact; echo "rc=$?"
nvm alias novee; echo "rc=$?"
nvm alias missing
