# id: INS-30
# The npm suffix comes from running npm --version, and is dropped when npm is missing.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# setup: printf '#!/bin/sh\necho 11.2.0\n' > "$NVM_DIR/versions/node/v24.15.0/lib/node_modules/npm/bin/npm-cli.js"
# setup: rm "$NVM_DIR/versions/node/v20.20.2/bin/npm"
nvm use 24.15.0; echo "rc=$?"
nvm use 20.20.2; echo "rc=$?"
