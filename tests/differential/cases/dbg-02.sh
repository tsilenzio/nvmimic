# id: DBG-02
# covers: nvm nvm:--version nvm:current nvm:debug nvm_command_info nvm_echo nvm_err
# covers: nvm_get_checksum_binary nvm_get_os nvm_grep nvm_has nvm_has_non_aliased nvm_is_alias
# covers: nvm_is_zsh nvm_ls_current nvm_sanitize_path nvm_version
# nvm debug shows an aliased program with what it is aliased to, and says when node is missing.
alias grep='grep -E'
SHELL=/bin/sh; export SHELL
nvm debug 2>&1 | command grep -E '^(grep|git|sed|which node): '; echo "rc=$?"
