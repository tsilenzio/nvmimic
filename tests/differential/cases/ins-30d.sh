# id: INS-30d
# An npm that is a shim from another package, as Corepack installs, is run to get its version.
# covers: nvm_print_npm_version nvm:use
# setup: nvm install 24.15.0
# setup: c="$HOME/shim"
# setup: mkdir -p "$c/bin" "$c/lib/node_modules/corepack/dist"
# setup: printf '#!/bin/sh\necho 10.1.0\n' > "$c/lib/node_modules/corepack/dist/npm.js"
# setup: chmod +x "$c/lib/node_modules/corepack/dist/npm.js"
# setup: printf '{"name": "corepack", "version": "0.34.0", "engines": {"node": ">=18"}}\n' > "$c/lib/node_modules/corepack/package.json"
# setup: ln -s ../lib/node_modules/corepack/dist/npm.js "$c/bin/npm"
PATH="$HOME/shim/bin:$PATH"
nvm use 24.15.0; echo "rc=$?"
