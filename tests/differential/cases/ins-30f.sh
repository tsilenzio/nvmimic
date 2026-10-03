# id: INS-30f
# With no npm anywhere on PATH, no npm version is shown.
# covers: nvm_print_npm_version nvm:use
# setup: nvm install 24.15.0
# setup: rm "$NVM_DIR/versions/node/v24.15.0/bin/npm"
nvm use 24.15.0; echo "rc=$?"
nvm exec 24.15.0 node --version; echo "rc=$?"
