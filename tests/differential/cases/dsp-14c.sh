# id: DSP-14c
# run stops its flag loop at --.
# setup: nvm install 20.20.2
# covers: nvm nvm:deactivate nvm:exec nvm:run nvm:use nvm_alias nvm_alias_path nvm_auto
# covers: nvm_change_path nvm_die_on_prefix nvm_echo nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_find_nvmrc nvm_find_project_dir nvm_find_up
# covers: nvm_get_os nvm_grep nvm_has nvm_has_system_iojs nvm_has_system_node nvm_hash_reset
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_valid_version nvm_is_version_installed
# covers: nvm_is_zsh nvm_ls nvm_ls_current nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_npm_version
# covers: nvm_process_parameters nvm_rc_version nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to
# covers: nvm_version_path
nvm run -- 20.20.2 app.js
