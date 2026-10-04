# id: LST-04a
# ls with alias and keyword patterns.
# setup: nvm install 22.20.0
# setup: nvm install 24.21.0
# setup: nvm install 20.20.2
# setup: nvm use 20.20.2
# covers: nvm nvm:deactivate nvm:ls nvm_alias nvm_alias_path nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_colors nvm_get_os nvm_grep nvm_has nvm_has_colors nvm_has_system_iojs
# covers: nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_print_alias_file nvm_print_color_code
# covers: nvm_print_versions nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
nvm ls default; echo "rc=$?"
nvm ls node; echo "rc=$?"
nvm ls current; echo "rc=$?"
nvm ls 'lts/*'; echo "rc=$?"
nvm ls lts/jod; echo "rc=$?"
nvm ls system
