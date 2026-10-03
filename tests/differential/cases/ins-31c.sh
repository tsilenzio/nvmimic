# id: INS-31c
# install-latest-npm without npm exits 2.
# setup: nvm install 24.15.0
# setup: rm "$NVM_DIR/versions/node/v24.15.0/bin/npm"
nvm use 24.15.0 >/dev/null
nvm install-latest-npm; echo "rc=$?"
