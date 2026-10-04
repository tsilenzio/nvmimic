# id: DSP-25
# covers: nvm nvm:alias nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_get_colors nvm_get_os nvm_has nvm_has_colors
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_ls nvm_ls_current
# covers: nvm_make_alias nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_print_color_code nvm_print_formatted_alias nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_path
# alias name validation and the missing-target warning.
nvm alias 'bad#name' 20.20.2
echo "rc=$?"
nvm alias 'bad#name'
echo "rc=$?"
nvm alias sub/name 20.20.2
echo "rc=$?"
nvm alias ghost 99.0.0
