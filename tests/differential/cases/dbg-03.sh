# id: DBG-03
# nvm debug with SHELL unset shows the shell's own complaint about the empty command.
# setup: nvm install 24.15.0
# covers: nvm nvm:--version nvm:current nvm:debug nvm_command_info nvm_echo nvm_err
# covers: nvm_get_checksum_binary nvm_get_os nvm_grep nvm_has nvm_has_non_aliased nvm_is_alias
# covers: nvm_is_zsh nvm_ls_current nvm_sanitize_path nvm_tree_contains_path nvm_version
# covers: nvm_version_dir
unset SHELL
nvm debug 2>&1 | command grep -B2 -A1 'shell version'; echo "rc=$?"
