# id: LST-10a
# alias with a prefix lists matches across all groups.
# setup: nvm install 22.20.0
# setup: nvm alias site 22
# covers: nvm nvm:alias nvm:deactivate nvm_alias nvm_alias_path nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_colors nvm_get_os nvm_grep nvm_has nvm_has_colors nvm_has_system_iojs
# covers: nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_list_aliases nvm_ls nvm_ls_current nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_print_alias_file nvm_print_alias_path
# covers: nvm_print_color_code nvm_print_formatted_alias nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_path
nvm alias s; echo "rc=$?"
nvm alias d; echo "rc=$?"
nvm alias lts/j
