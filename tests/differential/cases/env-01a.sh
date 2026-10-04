# id: ENV-01a
# covers: nvm_alias nvm_alias_path nvm_auto nvm_cd nvm_echo nvm_err nvm_find_nvmrc nvm_find_up
# covers: nvm_iojs_prefix nvm_is_zsh nvm_ls_current nvm_node_prefix nvm_normalize_lts
# covers: nvm_process_parameters nvm_rc_version nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_validate_implicit_alias nvm_version_dir
# With NVM_DIR unset, sourcing detects it from the script location and exports it.
script="$NVM_DIR/nvm.sh"
unset NVM_DIR
. "$script"
echo "rc=$?"
echo "NVM_DIR=${NVM_DIR-unset}"
sh -c 'echo "child NVM_DIR=${NVM_DIR-unset}"'
