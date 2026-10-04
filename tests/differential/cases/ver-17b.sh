# id: VER-17b
# An alias cycle resolves to infinity and which aborts with exit 8.
# setup: nvm install 22.20.0
# setup: nvm alias loopa loopb
# setup: nvm alias loopb loopa
# covers: nvm nvm:alias nvm:version nvm:which nvm_alias nvm_alias_path nvm_echo nvm_err
# covers: nvm_get_colors nvm_has nvm_has_colors nvm_iojs_prefix nvm_is_zsh nvm_list_aliases nvm_ls
# covers: nvm_ls_current nvm_node_prefix nvm_normalize_lts nvm_print_alias_file nvm_print_alias_path
# covers: nvm_print_color_code nvm_print_formatted_alias nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_tree_contains_path nvm_version nvm_version_dir
nvm alias loopa; echo "rc=$?"
nvm version loopa; echo "rc=$?"
nvm which loopa
