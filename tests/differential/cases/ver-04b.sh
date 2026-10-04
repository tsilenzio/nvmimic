# id: VER-04b
# ls node lists every installed version.
# setup: nvm install 22.20.0
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:ls nvm_echo nvm_ensure_version_prefix nvm_get_colors nvm_grep
# covers: nvm_has nvm_has_colors nvm_has_system_iojs nvm_has_system_node nvm_hash_reset
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh nvm_ls nvm_ls_current nvm_node_prefix
# covers: nvm_num_version_groups nvm_print_color_code nvm_print_versions nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path
# covers: nvm_version_dir
nvm ls node
