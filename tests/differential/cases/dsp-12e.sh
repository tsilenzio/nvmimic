# id: DSP-12e
# use exits 11 when NPM_CONFIG_PREFIX points outside NVM_DIR.
# setup: nvm install 20.20.2
# setup: nvm deactivate
NPM_CONFIG_PREFIX="$HOME/elsewhere" nvm use 20.20.2
