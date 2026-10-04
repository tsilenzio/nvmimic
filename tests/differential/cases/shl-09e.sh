# id: SHL-09e
# A non-default IFS does not break nvm and is restored, no helper variables leak.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# covers: nvm nvm:alias nvm:current nvm:deactivate nvm:ls nvm:use nvm_add_iojs_prefix nvm_alias
# covers: nvm_alias_path nvm_change_path nvm_die_on_prefix nvm_echo nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_find_project_dir nvm_get_colors nvm_get_os nvm_grep
# covers: nvm_has nvm_has_colors nvm_has_system_iojs nvm_has_system_node nvm_hash_reset
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_list_aliases
# covers: nvm_ls nvm_ls_current nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_normalize_version nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_alias_file
# covers: nvm_print_alias_path nvm_print_color_code nvm_print_default_alias
# covers: nvm_print_formatted_alias nvm_print_implicit_alias nvm_print_npm_version
# covers: nvm_print_versions nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
IFS=,
nvm use 20.20.2
nvm ls
nvm current
[ "$IFS" = , ] && echo "IFS kept"
IFS=' '
for v in DEFAULT_IFS EXIT_CODE VERSION NVM_USE_OUTPUT; do
  eval "[ -n \"\${$v+x}\" ]" && echo "leaked $v"
done
echo done
