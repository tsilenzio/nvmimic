# id: DSP-24
# alias flag handling, positionals and the empty-target unalias.
# setup: nvm install 20.20.2
# covers: nvm nvm:alias nvm:unalias nvm_alias nvm_alias_path nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_colors nvm_get_os nvm_has nvm_has_colors nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current
# covers: nvm_make_alias nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_print_alias_file nvm_print_color_code nvm_print_formatted_alias nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_tree_contains_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
nvm alias --foo
echo "rc=$?"
nvm alias -- --no-colors myalias 20.20.2 ignored
echo "rc=$?"
nvm alias myalias ""
