# id: INS-30c
# An npm earlier on PATH than the selected version's is the one reported.
# covers: nvm_print_npm_version nvm:use
# setup: nvm install 24.15.0
# setup: o="$HOME/other"
# setup: mkdir -p "$o/bin" "$o/lib/node_modules/npm/bin"
# setup: printf '#!/bin/sh\necho 9.8.7\n' > "$o/lib/node_modules/npm/bin/npm-cli.js"
# setup: chmod +x "$o/lib/node_modules/npm/bin/npm-cli.js"
# setup: printf '{"name": "npm", "version": "9.8.7", "engines": {"node": ">=18"}}\n' > "$o/lib/node_modules/npm/package.json"
# setup: ln -s ../lib/node_modules/npm/bin/npm-cli.js "$o/bin/npm"
nvm use 24.15.0; echo "rc=$?"
PATH="$HOME/other/bin:$PATH"
nvm use 24.15.0; echo "rc=$?"
