# id: INS-30e
# An npm that does not support the active node is run, and prints no version.
# covers: nvm_print_npm_version nvm:use
# setup: nvm install 24.15.0
# setup: d="$NVM_DIR/versions/node/v24.15.0/lib/node_modules/npm"
# setup: printf '#!/bin/sh\necho "npm does not support this node" >&2\nexit 1\n' > "$d/bin/npm-cli.js"
# setup: printf '{"name": "npm", "version": "12.0.0", "engines": {"node": ">=99.0.0"}}\n' > "$d/package.json"
nvm use 24.15.0; echo "rc=$?"
