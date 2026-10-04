# id: VER-10b
# lts/* is N/A when only an older patch of the LTS line is installed.
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:version nvm:which nvm_add_iojs_prefix nvm_alias nvm_alias_path
# covers: nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_os
# covers: nvm_grep nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_ls nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_print_alias_file nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_path
nvm version 'lts/*'; echo "rc=$?"
nvm which 'lts/*'
