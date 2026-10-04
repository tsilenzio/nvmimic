# id: VER-13
# Bare lts is not an alias.
# setup: nvm install 22.20.0
# covers: nvm nvm:deactivate nvm:use nvm:version nvm:which nvm_alias nvm_alias_path nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_grep
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_zsh nvm_ls nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_validate_implicit_alias nvm_version nvm_version_dir
nvm version lts; echo "rc=$?"
nvm which lts; echo "rc=$?"
nvm use lts
