# id: FIL-05a
# alias writes one file per alias holding the target.
# setup: nvm install 20.20.2
# covers: nvm nvm:alias nvm_alias nvm_alias_path nvm_echo nvm_ensure_version_prefix nvm_err
# covers: nvm_get_colors nvm_get_os nvm_has nvm_has_colors nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_ls nvm_ls_current nvm_make_alias nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_print_color_code nvm_print_formatted_alias
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_tree_contains_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
nvm alias foo 20.20.2
echo "rc=$?"
nvm alias team/bar 20
cat "$NVM_DIR/alias/foo"
cat "$NVM_DIR/alias/team/bar"
