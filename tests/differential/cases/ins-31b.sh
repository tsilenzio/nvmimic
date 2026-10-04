# id: INS-31b
# install-latest-npm picks the npm target by node version.
# setup: nvm install 14.21.3
# setup: nvm install 18.20.8
# setup: nvm install 20.20.2
# setup: nvm install 24.15.0
# covers: nvm nvm:install-latest-npm nvm:use nvm_alias nvm_alias_path nvm_change_path
# covers: nvm_die_on_prefix nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_find_project_dir nvm_get_os nvm_grep nvm_has nvm_hash_reset nvm_install_latest_npm
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_ls nvm_ls_current
# covers: nvm_match_version nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears
# covers: nvm_num_version_groups nvm_print_npm_version nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_tree_contains_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
for v in 14.21.3 18.20.8 20.20.2 24.15.0; do
  nvm use "$v" >/dev/null
  nvm install-latest-npm; echo "rc=$?"
done
