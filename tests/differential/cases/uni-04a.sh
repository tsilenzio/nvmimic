# id: UNI-04a
# uninstall of a version that is not installed reports it and exits 0.
# setup: nvm install 20.20.2
# covers: nvm nvm:uninstall nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_get_os nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_ls nvm_ls_current nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_tree_contains_path nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_path
nvm uninstall 22.20.0
