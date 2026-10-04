# id: SHL-30d
# covers: nvm nvm:alias nvm:deactivate nvm:install nvm:ls nvm:use nvm_add_iojs_prefix nvm_alias
# covers: nvm_alias_path nvm_change_path nvm_curl_libz_support nvm_curl_use_compression
# covers: nvm_curl_version nvm_die_on_prefix nvm_download nvm_echo nvm_ensure_default_set
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_project_dir
# covers: nvm_get_colors nvm_get_default_packages nvm_get_mirror nvm_get_os nvm_grep nvm_has
# covers: nvm_has_colors nvm_has_executable nvm_has_system_iojs nvm_has_system_node nvm_hash_reset
# covers: nvm_install_default_packages nvm_install_npm_if_needed nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_list_aliases nvm_ls nvm_ls_current nvm_ls_remote
# covers: nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_normalize_version nvm_npmrc_bad_news_bears
# covers: nvm_num_version_groups nvm_print_alias_file nvm_print_alias_path nvm_print_color_code
# covers: nvm_print_default_alias nvm_print_formatted_alias nvm_print_implicit_alias
# covers: nvm_print_npm_version nvm_print_versions nvm_remote_version nvm_remote_versions
# covers: nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path
# covers: nvm_tree_contains_path nvm_use_if_needed nvm_validate_implicit_alias nvm_validate_install
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to
# covers: nvm_version_path
# The install hook can be a shell function.
myhook() {
  echo "hook got: $1 $2 $3 $4"
  mkdir -p "$5/bin"
  printf '#!/bin/sh\necho v24.15.0\n' > "$5/bin/node"
  printf '#!/bin/sh\necho 10.9.0\n' > "$5/bin/npm"
  chmod +x "$5/bin/node" "$5/bin/npm"
}
NVM_INSTALL_THIRD_PARTY_HOOK=myhook
nvm install 24.15.0; echo "rc=$?"
failing() { echo "failing hook"; return 5; }
NVM_INSTALL_THIRD_PARTY_HOOK=failing
nvm install 20.20.2; echo "rc=$?"
unset NVM_INSTALL_THIRD_PARTY_HOOK
nvm ls; echo "rc=$?"
