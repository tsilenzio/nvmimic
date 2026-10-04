# id: RC-01b
# Discovery walks the logical PWD through a symlinked directory.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
# setup: mkdir -p real/proj logical && echo 20.20.2 > real/.nvmrc && echo 22.20.0 > logical/.nvmrc
# setup: ln -s ../real/proj logical/link
# covers: nvm nvm:which nvm_alias nvm_alias_path nvm_echo nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_find_nvmrc nvm_find_up nvm_get_os nvm_grep
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_ls nvm_node_prefix
# covers: nvm_normalize_lts nvm_num_version_groups nvm_process_nvmrc nvm_process_nvmrc_content
# covers: nvm_rc_version nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
cd logical/link || exit 99
nvm_find_nvmrc; echo "rc=$?"
nvm which
