# id: VER-06
# unstable is N/A for modern installs.
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:version nvm:which nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_grep
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_zsh nvm_ls nvm_node_prefix nvm_normalize_lts nvm_normalize_version
# covers: nvm_num_version_groups nvm_print_implicit_alias nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir
nvm version unstable; echo "rc=$?"
nvm which unstable
