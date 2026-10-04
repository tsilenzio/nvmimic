# id: SHL-09d
# Under bash set -E nvm works and errtrace stays on.
# shells: bash
# setup: nvm install 24.15.0
# covers: nvm nvm:current nvm:use nvm_alias nvm_alias_path nvm_change_path nvm_die_on_prefix
# covers: nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_find_project_dir nvm_get_os nvm_grep nvm_has nvm_hash_reset nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_version_installed nvm_ls nvm_ls_current nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_print_npm_version nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_tree_contains_path nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_path
set -E
nvm use 24.15.0
nvm current
case $- in *E*) echo "errtrace on" ;; *) echo "errtrace off" ;; esac
