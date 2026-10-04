# id: DSP-23c
# covers: nvm nvm:deactivate nvm:which nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_grep
# covers: nvm_has_system_iojs nvm_has_system_node nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh
# covers: nvm_ls nvm_node_prefix nvm_normalize_lts nvm_num_version_groups nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir
# which on a version that is not installed exits 1.
nvm which 18
