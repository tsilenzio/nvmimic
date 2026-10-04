# id: VER-20
# alias default lts/* stores and lists the typed text.
# setup: nvm install 24.21.0
# covers: nvm nvm:alias nvm_alias nvm_alias_path nvm_echo nvm_ensure_version_prefix nvm_err
# covers: nvm_get_colors nvm_get_os nvm_has nvm_has_colors nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_list_aliases nvm_ls nvm_ls_current nvm_make_alias
# covers: nvm_node_prefix nvm_normalize_lts nvm_num_version_groups nvm_print_alias_file
# covers: nvm_print_alias_path nvm_print_color_code nvm_print_formatted_alias nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_tree_contains_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
nvm alias default 'lts/*'; echo "rc=$?"
nvm alias default
