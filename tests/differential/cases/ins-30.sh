# id: INS-30
# The npm suffix comes from running npm --version, and is dropped when npm is missing.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# setup: printf '#!/bin/sh\necho 11.2.0\n' > "$NVM_DIR/versions/node/v24.15.0/lib/node_modules/npm/bin/npm-cli.js"
# setup: rm "$NVM_DIR/versions/node/v20.20.2/bin/npm"
# covers: nvm nvm:use nvm_alias nvm_alias_path nvm_change_path nvm_die_on_prefix nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_project_dir
# covers: nvm_get_os nvm_grep nvm_has nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_ls nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_npm_version nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
nvm use 24.15.0; echo "rc=$?"
nvm use 20.20.2; echo "rc=$?"
