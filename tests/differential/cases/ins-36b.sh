# id: INS-36b
# prefix or globalconfig in a builtin, global, user or project npmrc make nvm use exit 11.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# covers: nvm nvm:current nvm:install nvm:use nvm_alias nvm_alias_path nvm_change_path
# covers: nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version nvm_die_on_prefix
# covers: nvm_download nvm_echo nvm_ensure_default_set nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_find_project_dir nvm_get_mirror nvm_get_os nvm_grep
# covers: nvm_has nvm_has_executable nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current nvm_ls_remote
# covers: nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_print_alias_file nvm_print_npm_version nvm_remote_version nvm_remote_versions
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_sanitize_path nvm_strip_iojs_prefix
# covers: nvm_tree_contains_path nvm_validate_implicit_alias nvm_validate_install nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
d="$NVM_DIR/versions/node/v24.15.0"
printf 'prefix=/opt/x\n' > "$d/lib/node_modules/npm/npmrc"
nvm use 24.15.0; echo "rc=$?"
rm "$d/lib/node_modules/npm/npmrc"
mkdir -p "$d/etc" && printf 'globalconfig = /opt/x\n' > "$d/etc/npmrc"
nvm use 24.15.0; echo "rc=$?"
rm "$d/etc/npmrc"
printf 'prefix=/opt/x\n' > "$HOME/.npmrc"
nvm use 24.15.0; echo "rc=$?"
rm "$HOME/.npmrc"
printf '{}\n' > package.json && printf 'globalconfig=/opt/x\n' > .npmrc
nvm use 24.15.0; echo "rc=$?"
nvm install 24.15.0; echo "rc=$?"
nvm current
