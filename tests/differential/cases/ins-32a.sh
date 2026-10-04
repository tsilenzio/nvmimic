# id: INS-32a
# reinstall-packages and copy-packages usage, current-version and missing-system errors.
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:reinstall-packages nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_get_os nvm_grep nvm_has_system_iojs
# covers: nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_path
nvm reinstall-packages; echo "rc=$?"
nvm copy-packages 20 18; echo "rc=$?"
nvm reinstall-packages 24.15.0; echo "rc=$?"
nvm copy-packages 24; echo "rc=$?"
nvm reinstall-packages system; echo "rc=$?"
