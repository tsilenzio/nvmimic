# id: INS-18
# A reported success that fails post-install validation exits 1 without activating.
# setup: printf '#!/bin/sh\nmkdir -p "$5/bin" && printf "#!/bin/sh\\necho $1\\n" > "$5/bin/node" && chmod +x "$5/bin/node" && ln -s missing "$5/bin/npm"\n' > "$HOME/hook" && chmod +x "$HOME/hook"
# covers: nvm nvm:current nvm:install nvm_alias_path nvm_curl_libz_support nvm_curl_use_compression
# covers: nvm_curl_version nvm_download nvm_echo nvm_ensure_version_prefix nvm_err nvm_get_mirror
# covers: nvm_get_os nvm_grep nvm_has_executable nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls_current nvm_ls_remote nvm_ls_remote_index_tab
# covers: nvm_ls_remote_iojs nvm_make_alias nvm_node_prefix nvm_remote_version nvm_remote_versions
# covers: nvm_strip_iojs_prefix nvm_validate_implicit_alias nvm_validate_install nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to nvm_version_path
NVM_INSTALL_THIRD_PARTY_HOOK="$HOME/hook" nvm install 24.15.0; echo "rc=$?"
nvm current
