# id: SHL-07b
# A prefix in ~/.npmrc is reported on stderr while sourcing.
# setup: (nvm install 24.15.0)
# covers: nvm nvm:use nvm_alias nvm_alias_path nvm_auto nvm_change_path nvm_die_on_prefix nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_os nvm_grep nvm_has
# covers: nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version nvm_is_valid_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_print_alias_file nvm_process_parameters nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_sanitize_path nvm_strip_iojs_prefix nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
echo 'prefix=/opt/elsewhere' > "$HOME/.npmrc"
. "$NVM_DIR/nvm.sh"
echo "rc=$?"
