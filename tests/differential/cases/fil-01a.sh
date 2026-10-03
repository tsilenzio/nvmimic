# id: FIL-01a
# nvm-exec selects NODE_VERSION.
# setup: (nvm install 24.15.0)
# setup: (nvm install 20.20.2)
NODE_VERSION=20.20.2 "$NVM_DIR/nvm-exec" node --version
echo "rc=$?"
