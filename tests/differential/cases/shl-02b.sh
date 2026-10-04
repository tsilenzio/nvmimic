# id: SHL-02b
# Unknown source-time arguments are ignored and the default mode is use.
# setup: (nvm install 24.15.0)
# covers: nvm nvm:current nvm:use nvm_alias nvm_alias_path nvm_auto nvm_change_path
# covers: nvm_die_on_prefix nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_find_project_dir nvm_get_os nvm_grep nvm_has nvm_hash_reset nvm_iojs_prefix
# covers: nvm_is_iojs_version nvm_is_valid_version nvm_is_version_installed nvm_is_zsh nvm_ls
# covers: nvm_ls_current nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_alias_file
# covers: nvm_process_parameters nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_tree_contains_path nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
. "$NVM_DIR/nvm.sh" --bogus extra
echo "rc=$?"
node --version
nvm current
