# id: SHL-03c
# A default alias resolving to N/A activates nothing and ignores .nvmrc.
# setup: (nvm install 20.20.2 && nvm alias default 18.20.8)
# setup: echo 20.20.2 > .nvmrc
# covers: nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_auto nvm_echo nvm_ensure_version_prefix
# covers: nvm_err nvm_get_os nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh
# covers: nvm_ls nvm_ls_current nvm_node_prefix nvm_normalize_lts nvm_num_version_groups
# covers: nvm_print_alias_file nvm_process_parameters nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_path
. "$NVM_DIR/nvm.sh"
echo "rc=$?"
command -v node || echo "no node"
