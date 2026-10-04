# id: LST-03
# covers: nvm nvm:alias nvm:deactivate nvm:ls nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_get_colors nvm_grep nvm_has nvm_has_colors
# covers: nvm_has_system_iojs nvm_has_system_node nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh
# covers: nvm_list_aliases nvm_ls nvm_ls_current nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_print_alias_path nvm_print_color_code nvm_print_default_alias
# covers: nvm_print_formatted_alias nvm_print_implicit_alias nvm_print_versions nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir
# ls with nothing installed prints N/A, the alias section, exit 3.
nvm ls
