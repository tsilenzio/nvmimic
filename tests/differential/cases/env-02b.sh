# id: ENV-02b
# covers: nvm_alias nvm_alias_path nvm_auto nvm_echo nvm_err nvm_find_nvmrc nvm_find_up
# covers: nvm_iojs_prefix nvm_is_zsh nvm_ls_current nvm_node_prefix nvm_normalize_lts
# covers: nvm_process_parameters nvm_rc_version nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_validate_implicit_alias nvm_version_dir
# Several trailing slashes are all stripped with one warning.
script="$NVM_DIR/nvm.sh"
NVM_DIR="$NVM_DIR///"
. "$script"
echo "NVM_DIR=$NVM_DIR"
