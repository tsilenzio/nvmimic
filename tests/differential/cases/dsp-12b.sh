# id: DSP-12b
# use on an alias loop exits 8.
# setup: nvm alias loopa loopb
# setup: nvm alias loopb loopa
# covers: nvm nvm:use nvm_alias nvm_alias_path nvm_echo nvm_err nvm_iojs_prefix nvm_is_zsh nvm_ls
# covers: nvm_match_version nvm_node_prefix nvm_normalize_lts nvm_print_alias_file nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_version nvm_version_dir
nvm use loopa
