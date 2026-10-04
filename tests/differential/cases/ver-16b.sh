# id: VER-16b
# current as a spec with no active version.
# setup: nvm install 22.20.0
# setup: nvm deactivate
# covers: nvm nvm:deactivate nvm:ls nvm:version nvm:which nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_colors nvm_get_os
# covers: nvm_grep nvm_has nvm_has_colors nvm_has_system_iojs nvm_has_system_node nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current
# covers: nvm_node_prefix nvm_normalize_lts nvm_num_version_groups nvm_print_color_code
# covers: nvm_print_versions nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
nvm version current; echo "rc=$?"
nvm which current; echo "rc=$?"
nvm ls current
