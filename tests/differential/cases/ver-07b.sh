# id: VER-07b
# latest follows a user alias named latest.
# setup: nvm install 24.15.0
# setup: nvm install 22.20.0
# setup: nvm alias latest 22.20.0
# covers: nvm nvm:version nvm:which nvm_alias nvm_alias_path nvm_echo nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_get_os nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_print_alias_file nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_path
nvm version latest; echo "rc=$?"
nvm which latest
