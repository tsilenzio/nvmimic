# id: FIL-01d
# nvm-exec with an uninstalled NODE_VERSION fails with 127.
# setup: (nvm install 24.15.0)
NODE_VERSION=18.20.8 "$NVM_DIR/nvm-exec" node --version
echo "rc=$?"
