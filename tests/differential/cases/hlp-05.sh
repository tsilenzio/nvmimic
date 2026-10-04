# id: HLP-05
# The commands the README fish hook runs through bass, approximated in a bash child.
# shells: bash
# setup: (nvm install 24.15.0)
# setup: (nvm install 20.20.2)
# covers: nvm nvm:use nvm:version nvm_alias nvm_alias_path nvm_auto nvm_change_path
# covers: nvm_die_on_prefix nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_find_nvmrc nvm_find_project_dir nvm_find_up nvm_get_os nvm_grep nvm_has nvm_hash_reset
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_ls
# covers: nvm_ls_current nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_alias_file nvm_print_npm_version
# covers: nvm_process_parameters nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
echo 20.20.2 > .nvmrc
bash -c '. "$NVM_DIR/nvm.sh" --no-use ; nvm_find_nvmrc'
bash -c '. "$NVM_DIR/nvm.sh" --no-use ; nvm version default'
bash -c '. "$NVM_DIR/nvm.sh" --no-use ; nvm version'
bash -c '. "$NVM_DIR/nvm.sh" --no-use ; nvm version "$(cat .nvmrc)"'
bash -c '. "$NVM_DIR/nvm.sh" --no-use ; nvm use 20.20.2 >/dev/null; echo "$PATH"'
