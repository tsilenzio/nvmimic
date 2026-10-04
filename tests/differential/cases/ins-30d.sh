# id: INS-30d
# An npm that is a shim from another package, as Corepack installs, is run to get its version.
# setup: nvm install 24.15.0
# setup: c="$HOME/shim"
# setup: mkdir -p "$c/bin" "$c/lib/node_modules/corepack/dist"
# setup: printf '#!/bin/sh\necho 10.1.0\n' > "$c/lib/node_modules/corepack/dist/npm.js"
# setup: chmod +x "$c/lib/node_modules/corepack/dist/npm.js"
# setup: printf '{"name": "corepack", "version": "0.34.0", "engines": {"node": ">=18"}}\n' > "$c/lib/node_modules/corepack/package.json"
# setup: ln -s ../lib/node_modules/corepack/dist/npm.js "$c/bin/npm"
# covers: nvm nvm:use nvm_alias nvm_alias_path nvm_change_path nvm_die_on_prefix nvm_echo
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_project_dir
# covers: nvm_get_os nvm_grep nvm_has nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version
# covers: nvm_is_version_installed nvm_ls nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_npm_version nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_validate_implicit_alias nvm_version
# covers: nvm_version_dir nvm_version_greater nvm_version_path
PATH="$HOME/shim/bin:$PATH"
nvm use 24.15.0; echo "rc=$?"
