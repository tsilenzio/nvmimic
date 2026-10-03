# id: INS-30b
# An npm upgraded in place reports its new version.
# covers: nvm_print_npm_version nvm:use
# setup: nvm install 24.15.0
# setup: d="$NVM_DIR/versions/node/v24.15.0/lib/node_modules/npm"
# setup: printf '#!/bin/sh\necho 11.2.0\n' > "$d/bin/npm-cli.js"
# setup: printf '{"name": "npm", "version": "11.2.0", "engines": {"node": "^20.17.0 || >=22.9.0"}}\n' > "$d/package.json"
nvm use 24.15.0; echo "rc=$?"
