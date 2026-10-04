# id: LST-04c
# ls patterns with a comment or newline are parsed as .nvmrc content.
# setup: nvm install 22.20.0
# setup: nvm install 24.21.0
# covers: nvm nvm:deactivate nvm:ls nvm_alias nvm_alias_path nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_colors nvm_grep nvm_has nvm_has_colors nvm_has_system_iojs
# covers: nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh nvm_ls
# covers: nvm_ls_current nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_nvmrc_invalid_msg nvm_print_color_code nvm_print_versions nvm_process_nvmrc_content
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_tree_contains_path nvm_validate_implicit_alias nvm_version_dir
# covers: nvm_wrap_with_color_code
nvm ls '22 # pinned'; echo "rc=$?"
nvm ls "$(printf '# header\n24')"; echo "rc=$?"
nvm ls "$(printf '22\n24')"
