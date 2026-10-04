# id: VER-11b
# Uppercase LTS names fail, with the lowercase error where nvm surfaces it.
# setup: nvm install 22.20.0
# covers: nvm nvm:alias nvm:deactivate nvm:install nvm:version nvm:which nvm_alias nvm_alias_path
# covers: nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version nvm_download nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_get_mirror nvm_get_os
# covers: nvm_grep nvm_has_executable nvm_has_system_iojs nvm_has_system_node nvm_hash_reset
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh nvm_list_aliases nvm_ls nvm_ls_current
# covers: nvm_ls_remote nvm_ls_remote_index_tab nvm_make_alias nvm_node_prefix nvm_normalize_lts
# covers: nvm_num_version_groups nvm_remote_version nvm_remote_versions nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir
# covers: nvm_version_greater_than_or_equal_to
nvm version lts/Jod; echo "rc=$?"
nvm which lts/JOD; echo "rc=$?"
nvm alias lts/Jod; echo "rc=$?"
nvm install lts/Jod
