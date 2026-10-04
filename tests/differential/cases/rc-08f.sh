# id: RC-08f
# .nvmrc holding the selector 22 is honored.
# setup: nvm install 22.20.0
# setup: nvm install 24.21.0
# setup: nvm alias myalias 22.20.0
# setup: echo '22' > .nvmrc
# covers: nvm nvm:deactivate nvm:use nvm:which nvm_alias nvm_alias_path nvm_change_path
# covers: nvm_die_on_prefix nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_find_nvmrc nvm_find_project_dir nvm_find_up nvm_get_os nvm_grep nvm_has
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_npm_version nvm_process_nvmrc
# covers: nvm_process_nvmrc_content nvm_rc_version nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
nvm which; echo "rc=$?"
nvm use
