# id: VER-17a
# Aliases chain through other aliases.
# setup: nvm install 22.20.0
# setup: nvm alias inner 22.20.0
# setup: nvm alias outer inner
# covers: nvm nvm:alias nvm:version nvm:which nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_colors nvm_get_os
# covers: nvm_has nvm_has_colors nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed
# covers: nvm_is_zsh nvm_list_aliases nvm_ls nvm_ls_current nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_print_alias_file nvm_print_alias_path nvm_print_color_code
# covers: nvm_print_formatted_alias nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_tree_contains_path nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_path
nvm version outer; echo "rc=$?"
nvm which outer; echo "rc=$?"
nvm alias outer
