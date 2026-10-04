# id: LST-19
# which not-installed message for plain and alias specs.
# setup: nvm install 24.15.0
# setup: nvm alias old 18.20.8
# covers: nvm nvm:deactivate nvm:which nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_os nvm_grep
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_print_alias_file nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
nvm which 20.20.2; echo "rc=$?"
nvm which old; echo "rc=$?"
nvm which lts/iron; echo "rc=$?"
nvm which 'lts/*'
