# id: VER-05b
# covers: nvm nvm:deactivate nvm:ls-remote nvm_echo nvm_ensure_version_prefix nvm_err nvm_get_colors
# covers: nvm_grep nvm_has nvm_has_colors nvm_has_system_iojs nvm_has_system_node nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_zsh nvm_ls nvm_ls_current nvm_node_prefix
# covers: nvm_num_version_groups nvm_print_color_code nvm_print_versions nvm_remote_versions
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_validate_implicit_alias nvm_version_dir
# ls-remote stable prints N/A and exits 3.
nvm ls-remote stable
