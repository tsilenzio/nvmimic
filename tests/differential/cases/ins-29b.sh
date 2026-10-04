# id: INS-29b
# Non-WSL Windows cannot build from source, exit 87.
# setup: mkdir "$HOME/fake" && printf '#!/bin/sh\nif [ "$1" = -a ]; then echo "MINGW64_NT-10.0 host 3.4 x86_64 Msys"; else /usr/bin/uname "$@"; fi\n' > "$HOME/fake/uname" && chmod +x "$HOME/fake/uname"
# covers: nvm nvm:install nvm_acquire_install_lock nvm_alias_path nvm_cache_dir
# covers: nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version nvm_download nvm_echo
# covers: nvm_ensure_version_prefix nvm_err nvm_get_mirror nvm_get_os nvm_grep nvm_has_executable
# covers: nvm_install_lock_name nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed
# covers: nvm_is_zsh nvm_ls_remote nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias
# covers: nvm_node_prefix nvm_release_install_lock nvm_remote_version nvm_remote_versions
# covers: nvm_strip_iojs_prefix nvm_validate_implicit_alias nvm_version_dir nvm_version_greater
# covers: nvm_version_greater_than_or_equal_to nvm_version_path
PATH="$HOME/fake:$PATH"
export NVM_MAKE_JOBS=1
nvm install -s 24.15.0; echo "rc=$?"
