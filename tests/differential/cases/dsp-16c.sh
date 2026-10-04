# id: DSP-16c
# exec stops its flag loop at -- and leaves it in the arguments.
# setup: nvm install 20.20.2
# covers: nvm nvm:deactivate nvm:exec nvm:use nvm_alias nvm_alias_path nvm_auto nvm_change_path
# covers: nvm_die_on_prefix nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_find_nvmrc nvm_find_project_dir nvm_find_up nvm_get_os nvm_grep nvm_has
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_valid_version nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current
# covers: nvm_match_version nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears
# covers: nvm_num_version_groups nvm_print_npm_version nvm_process_parameters nvm_rc_version
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_tree_contains_path nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
nvm exec -- node app.js
