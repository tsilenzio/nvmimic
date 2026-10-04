# id: ENV-05c
# A preset NVM_COLORS does not change non-TTY listing output.
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:ls nvm_alias nvm_alias_path nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_colors nvm_grep nvm_has nvm_has_colors nvm_has_system_iojs
# covers: nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh nvm_ls
# covers: nvm_ls_current nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_print_color_code nvm_print_versions nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias
# covers: nvm_version_dir
NVM_COLORS=rgbcy
export NVM_COLORS
nvm ls 24
