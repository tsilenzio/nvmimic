# id: LST-07a
# Implicit alias lines carry the default suffix.
# setup: nvm install 24.15.0
# covers: nvm nvm:alias nvm:deactivate nvm_alias nvm_alias_path nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_colors nvm_grep nvm_has nvm_has_colors nvm_has_system_iojs
# covers: nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh
# covers: nvm_list_aliases nvm_ls nvm_ls_current nvm_node_prefix nvm_normalize_lts
# covers: nvm_normalize_version nvm_num_version_groups nvm_print_alias_path nvm_print_color_code
# covers: nvm_print_default_alias nvm_print_formatted_alias nvm_print_implicit_alias
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_tree_contains_path nvm_validate_implicit_alias nvm_version nvm_version_dir
nvm alias iojs; echo "rc=$?"
nvm alias node; echo "rc=$?"
nvm alias stable; echo "rc=$?"
nvm alias unstable
