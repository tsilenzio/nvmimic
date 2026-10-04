# id: LST-18
# which prints the installed node binary path.
# setup: nvm install 22.20.0
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:which nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_os nvm_grep
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_print_alias_file nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
nvm which 22; echo "rc=$?"
nvm which default; echo "rc=$?"
nvm which v24.15.0
