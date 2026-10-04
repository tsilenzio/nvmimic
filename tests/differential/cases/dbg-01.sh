# id: DBG-01
# nvm debug reports the shell, the programs it finds and the active version, and exits 42.
# setup: nvm install 24.15.0
# covers: nvm nvm:--version nvm:current nvm:debug nvm_command_info nvm_echo nvm_err
# covers: nvm_get_checksum_binary nvm_get_os nvm_grep nvm_has nvm_has_non_aliased nvm_is_alias
# covers: nvm_is_zsh nvm_ls_current nvm_sanitize_path nvm_tree_contains_path nvm_version
# covers: nvm_version_dir
SHELL=/bin/sh; export SHELL
nvm debug; echo "rc=$?"
