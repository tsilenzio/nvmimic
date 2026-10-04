# id: DBG-04
# covers: nvm nvm:--version nvm:current nvm:debug nvm_command_info nvm_echo nvm_err
# covers: nvm_get_checksum_binary nvm_get_os nvm_grep nvm_has nvm_has_non_aliased nvm_is_alias
# covers: nvm_is_zsh nvm_ls_current nvm_sanitize_path nvm_version
# nvm debug with no npm shows what the shell says about the missing command.
SHELL=/bin/sh; export SHELL
nvm debug 2>&1 | command grep -E '^(nvm current|which [a-z]+|npm [a-z -]+): '; echo "rc=$?"
