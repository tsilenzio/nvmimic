# id: DSP-27
# version reads only its first argument, empty means current, flags give N/A.
# setup: nvm install 20.20.2
# covers: nvm nvm:deactivate nvm:version nvm_alias nvm_alias_path nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_grep nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_zsh nvm_ls nvm_ls_current nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir
nvm version
echo "rc=$?"
nvm version ""
echo "rc=$?"
nvm version 20 22
echo "rc=$?"
nvm version --lts
