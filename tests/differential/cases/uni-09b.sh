# id: UNI-09b
# uninstall removes a version directory nvm did not create itself.
# setup: nvm install 22.20.0
# setup: mkdir -p "$NVM_DIR/versions/node/v18.20.8/bin" "$NVM_DIR/versions/node/v18.20.8/lib/node_modules/npm/bin"
# setup: printf '#!/bin/sh\necho v18.20.8\n' >"$NVM_DIR/versions/node/v18.20.8/bin/node"
# setup: printf '#!/bin/sh\necho 10.0.0\n' >"$NVM_DIR/versions/node/v18.20.8/lib/node_modules/npm/bin/npm-cli.js"
# setup: chmod +x "$NVM_DIR/versions/node/v18.20.8/bin/node" "$NVM_DIR/versions/node/v18.20.8/lib/node_modules/npm/bin/npm-cli.js"
# setup: ln -s ../lib/node_modules/npm/bin/npm-cli.js "$NVM_DIR/versions/node/v18.20.8/bin/npm"
nvm uninstall 18.20.8
echo "rc=$?"
ls "$NVM_DIR/versions/node"
