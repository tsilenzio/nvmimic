# id: FIL-01e
# nvm-exec resets NVM_CD_FLAGS and sets NVM_BIN for the child.
# setup: (nvm install 20.20.2)
NVM_CD_FLAGS=-x NODE_VERSION=20.20.2 "$NVM_DIR/nvm-exec" sh -c 'echo "NVM_CD_FLAGS=[$NVM_CD_FLAGS]"; echo "$NVM_BIN"; echo "NODE_VERSION=$NODE_VERSION"'
