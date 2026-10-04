# id: SHL-08
# Sourcing and calling nvm leave the caller's zsh options unchanged.
# shells: zsh
# setup: (nvm install 24.15.0)
# covers: nvm nvm:alias nvm:current nvm:deactivate nvm:ls nvm:use nvm_add_iojs_prefix nvm_alias
# covers: nvm_alias_path nvm_auto nvm_change_path nvm_die_on_prefix nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_project_dir
# covers: nvm_get_colors nvm_get_os nvm_grep nvm_has nvm_has_colors nvm_has_system_iojs
# covers: nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_valid_version nvm_is_version_installed nvm_is_zsh nvm_list_aliases nvm_ls
# covers: nvm_ls_current nvm_match_version nvm_node_prefix nvm_normalize_lts nvm_normalize_version
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_alias_file nvm_print_alias_path
# covers: nvm_print_color_code nvm_print_default_alias nvm_print_formatted_alias
# covers: nvm_print_implicit_alias nvm_print_versions nvm_process_parameters nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
before=$(setopt)
. "$NVM_DIR/nvm.sh"
nvm use 20.20.2 >/dev/null 2>&1
nvm ls >/dev/null
nvm current >/dev/null
after=$(setopt)
[ "$before" = "$after" ] && echo "options unchanged"
setopt
