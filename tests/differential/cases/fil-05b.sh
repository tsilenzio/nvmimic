# id: FIL-05b
# Comments and blank lines in a hand-written alias file are ignored.
# setup: nvm install 20.20.2
# setup: nvm install 24.15.0
# covers: nvm nvm:alias nvm:use nvm:version nvm_alias nvm_alias_path nvm_change_path
# covers: nvm_die_on_prefix nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err
# covers: nvm_find_project_dir nvm_get_colors nvm_get_os nvm_grep nvm_has nvm_has_colors
# covers: nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh
# covers: nvm_list_aliases nvm_ls nvm_ls_current nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_alias_file nvm_print_alias_path
# covers: nvm_print_color_code nvm_print_formatted_alias nvm_print_npm_version nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_tree_contains_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
mkdir -p "$NVM_DIR/alias"
printf '# pinned for ci\n\n20.20.2\n' > "$NVM_DIR/alias/ci"
nvm version ci
nvm use ci
nvm alias ci
