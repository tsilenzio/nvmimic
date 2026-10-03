# id: INS-36a
# PREFIX and any-case NPM_CONFIG_PREFIX outside NVM_DIR make nvm use exit 11 and deactivate.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
export PREFIX=/opt/elsewhere
nvm use 24.15.0; echo "rc=$?"
nvm current
unset PREFIX
export npm_config_prefix=/opt/elsewhere
nvm use 24.15.0; echo "rc=$?"
nvm current
unset npm_config_prefix
export NPM_CONFIG_PREFIX="$NVM_DIR/versions/node/v24.15.0"
nvm use 24.15.0; echo "rc=$?"
unset NPM_CONFIG_PREFIX
