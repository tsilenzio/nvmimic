# id: VER-14
# Custom alias names can be set, listed and used as specs.
# setup: nvm install 22.20.0
# setup: nvm install 24.15.0
# covers: nvm nvm:alias nvm:deactivate nvm:use nvm:version nvm:which nvm_alias nvm_alias_path
# covers: nvm_change_path nvm_die_on_prefix nvm_echo nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_find_project_dir nvm_get_colors nvm_get_os nvm_grep
# covers: nvm_has nvm_has_colors nvm_has_system_iojs nvm_has_system_node nvm_hash_reset
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_list_aliases
# covers: nvm_ls nvm_ls_current nvm_make_alias nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_alias_file nvm_print_alias_path
# covers: nvm_print_color_code nvm_print_formatted_alias nvm_print_npm_version nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
nvm alias myapp 22; echo "rc=$?"
nvm alias myapp; echo "rc=$?"
nvm version myapp; echo "rc=$?"
nvm which myapp; echo "rc=$?"
nvm use myapp
