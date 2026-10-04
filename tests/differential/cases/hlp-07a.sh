# id: HLP-07a
# nvm_ls lists installed versions, optionally filtered.
# setup: (nvm install 24.15.0)
# setup: (nvm install 20.20.2)
# setup: (nvm install 18.20.8)
# covers: nvm nvm:deactivate nvm_alias nvm_alias_path nvm_echo nvm_ensure_version_prefix nvm_err
# covers: nvm_grep nvm_has_system_iojs nvm_has_system_node nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_zsh nvm_ls nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_validate_implicit_alias nvm_version_dir
nvm_ls
echo "rc=$?"
nvm_ls 20
nvm_ls 99
echo "rc=$?"
