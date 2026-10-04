# id: FIL-05d
# A missing alias resolves to N/A and fails use.
# setup: nvm install 20.20.2
# covers: nvm nvm:alias nvm:deactivate nvm:use nvm:version nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_grep nvm_has
# covers: nvm_has_colors nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_zsh nvm_list_aliases nvm_ls nvm_ls_current nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_num_version_groups nvm_print_alias_path
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_tree_contains_path nvm_validate_implicit_alias nvm_version nvm_version_dir
nvm version nope
echo "rc=$?"
nvm use nope
echo "rc=$?"
nvm alias nope
echo "rc=$?"
