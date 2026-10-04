# id: RC-02f
# A line starting with = counts as the bare version line.
# setup: nvm install 22.20.0
# setup: printf '=22.20.0\n' > .nvmrc
# covers: nvm nvm:deactivate nvm:which nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_nvmrc nvm_find_up
# covers: nvm_grep nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_zsh nvm_ls nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_process_nvmrc nvm_process_nvmrc_content nvm_rc_version
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir
nvm which
