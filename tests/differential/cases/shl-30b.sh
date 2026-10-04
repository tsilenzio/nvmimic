# id: SHL-30b
# A replaced function is called from inside nvm use, and what nvm use changes stays in the shell.
# shells: bash sh zsh
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# covers: nvm nvm:current nvm:use nvm_alias nvm_alias_path nvm_change_path nvm_die_on_prefix
# covers: nvm_echo nvm_ensure_version_prefix nvm_err nvm_find_project_dir nvm_get_os nvm_grep
# covers: nvm_has nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_ls
# covers: nvm_ls_current nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_npm_version nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_tree_contains_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
nvm_ensure_version_installed() { echo "asked about: $*"; return 0; }
nvm use 20.20.2; echo "rc=$?"
echo "bin=${NVM_BIN-unset}"
nvm_print_npm_version() { printf ' (my npm line)'; }
nvm use 24.15.0; echo "rc=$?"
nvm current; echo "rc=$?"
