# id: VER-16a
# current is accepted as a spec while a version is active.
# setup: nvm install 24.15.0
# setup: nvm install 22.20.0
# setup: nvm use 22.20.0
# covers: nvm nvm:deactivate nvm:exec nvm:ls nvm:run nvm:use nvm:version nvm:which nvm_alias
# covers: nvm_alias_path nvm_auto nvm_change_path nvm_die_on_prefix nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_project_dir
# covers: nvm_get_colors nvm_get_os nvm_grep nvm_has nvm_has_colors nvm_has_system_iojs
# covers: nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_print_color_code nvm_print_npm_version nvm_print_versions nvm_process_parameters
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_tree_contains_path nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_path
nvm version current; echo "rc=$?"
nvm which current; echo "rc=$?"
nvm ls current; echo "rc=$?"
nvm exec current node --version; echo "rc=$?"
nvm run current --version; echo "rc=$?"
nvm use current
