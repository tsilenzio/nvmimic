# id: SHL-15a
# NVM_SYMLINK_CURRENT=true points $NVM_DIR/current at the version in use.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
NVM_SYMLINK_CURRENT=true
export NVM_SYMLINK_CURRENT
nvm use 24.15.0 >/dev/null
readlink "$NVM_DIR/current"
nvm use 20.20.2 >/dev/null
readlink "$NVM_DIR/current"
"$NVM_DIR/current/bin/node" --version
