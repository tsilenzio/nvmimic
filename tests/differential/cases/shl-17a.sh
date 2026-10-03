# id: SHL-17a
# deactivate strips nvm dirs from PATH, MANPATH and NODE_PATH and unsets NVM_BIN and NVM_INC.
# setup: nvm install 24.15.0
NODE_PATH="$NVM_DIR/versions/node/v24.15.0/lib/node_modules:/opt/lib"
export NODE_PATH
nvm deactivate
echo "rc=$?"
echo "NVM_BIN=${NVM_BIN-unset} NVM_INC=${NVM_INC-unset}"
command -v node || echo "no node"
