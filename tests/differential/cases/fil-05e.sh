# id: FIL-05e
# An unreadable alias file is reported.
# setup: nvm install 20.20.2
# covers: nvm nvm:alias nvm:deactivate nvm:use nvm:version nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_colors nvm_get_os
# covers: nvm_grep nvm_has nvm_has_colors nvm_has_system_iojs nvm_has_system_node nvm_hash_reset
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_list_aliases
# covers: nvm_ls nvm_ls_current nvm_make_alias nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_print_alias_path nvm_print_color_code nvm_print_formatted_alias
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_tree_contains_path nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_path
nvm alias locked 20.20.2 >/dev/null
chmod 000 "$NVM_DIR/alias/locked"
nvm version locked
echo "rc=$?"
nvm use locked
echo "rc=$?"
nvm alias locked
echo "rc=$?"
chmod 644 "$NVM_DIR/alias/locked"
