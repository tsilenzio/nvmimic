# id: VER-01
# Exact versions with and without the v prefix.
# setup: nvm install 24.15.0
# covers: nvm nvm:version nvm:which nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_os nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_version_installed nvm_ls nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
nvm version v24.15.0; echo "rc=$?"
nvm version 24.15.0; echo "rc=$?"
nvm which 24.15.0; echo "rc=$?"
nvm version 24.21.0; echo "rc=$?"
nvm which v24.15.0
