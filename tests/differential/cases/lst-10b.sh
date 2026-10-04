# id: LST-10b
# alias lts/<name> prints the raw file content.
# setup: nvm install 22.20.0
# covers: nvm nvm:alias nvm_alias nvm_alias_path nvm_echo nvm_is_zsh nvm_list_aliases nvm_ls_current
# covers: nvm_normalize_lts nvm_print_alias_file nvm_tree_contains_path nvm_version_dir
nvm alias lts/iron; echo "rc=$?"
nvm alias 'lts/*'
