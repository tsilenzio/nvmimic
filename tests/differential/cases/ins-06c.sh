# id: INS-06c
# --offline not-found wording points at nvm ls, exit 3.
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:install nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_cache_dir
# covers: nvm_echo nvm_ensure_version_prefix nvm_err nvm_get_arch nvm_get_os nvm_grep
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_cached nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_offline_version nvm_print_alias_file nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_path
nvm install --offline 99; echo "rc=$?"
nvm install --offline 20; echo "rc=$?"
nvm install --offline --lts=hydrogen 18; echo "rc=$?"
