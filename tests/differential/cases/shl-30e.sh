# id: SHL-30e
# NVMIMIC_NVM_CODE set before nvm.sh is sourced makes a replaced function count in every shell, dash included.
# setup: nvm install 24.15.0
# covers: nvm nvm:current nvm:use nvm_auto nvm_change_path nvm_die_on_prefix nvm_echo
# covers: nvm_ensure_version_installed nvm_find_project_dir nvm_get_os nvm_grep nvm_has
# covers: nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh
# covers: nvm_ls_current nvm_match_version nvm_node_prefix nvm_npmrc_bad_news_bears
# covers: nvm_print_npm_version nvm_process_parameters nvm_tree_contains_path nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
NVMIMIC_NVM_CODE=1
. "$NVM_DIR/nvm.sh" --no-use
nvm_ls() { echo "v9.9.9"; echo "my nvm_ls got: $*" >&2; }
nvm_version 22; echo "rc=$?"
unset NVMIMIC_NVM_CODE
nvm_ls() { echo v24.15.0; }
nvm use 24.15.0; echo "rc=$?"
nvm current; echo "rc=$?"
