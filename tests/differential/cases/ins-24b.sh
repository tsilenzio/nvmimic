# id: INS-24b
# A failing install-latest-npm sets the install exit code.
# setup: nvm install 24.15.0
# setup: rm "$NVM_DIR/versions/node/v24.15.0/bin/npm"
nvm install --latest-npm 24.15.0; echo "rc=$?"
