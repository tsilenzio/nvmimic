# id: VER-08a
# covers: nvm nvm:deactivate nvm:version nvm:which nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_grep nvm_has_system_iojs nvm_has_system_node
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh nvm_ls nvm_node_prefix nvm_normalize_lts
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir
# system with no system node.
nvm version system; echo "rc=$?"
nvm which system
