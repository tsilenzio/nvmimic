# id: INS-32b
# reinstall-packages reinstalls listed globals and links linked ones.
# setup: nvm install 20.20.2
# setup: nvm install 24.15.0
# setup: mkdir "$HOME/mylink"
# setup: printf '#!/bin/sh\nif [ "$1" = list ]; then printf "%%s\\n" "/x/lib" "+-- corepack@0.29.0" "+-- npm@10.9.0" "+-- typescript@5.6.2" "+-- prettier@3.3.3" "+-- mylink@1.0.0 -> %s"; else echo "npm20: $*"; fi\n' "$HOME/mylink" > "$NVM_DIR/versions/node/v20.20.2/lib/node_modules/npm/bin/npm-cli.js"
# covers: nvm nvm:deactivate nvm:reinstall-packages nvm:use nvm_alias nvm_alias_path nvm_cd
# covers: nvm_change_path nvm_die_on_prefix nvm_echo nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_find_project_dir nvm_get_os nvm_grep nvm_has
# covers: nvm_has_system_iojs nvm_has_system_node nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npm_global_modules nvm_npmrc_bad_news_bears
# covers: nvm_num_version_groups nvm_print_npm_version nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_path
nvm reinstall-packages 20; echo "rc=$?"
nvm copy-packages v20.20.2; echo "rc=$?"
