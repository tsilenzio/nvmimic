# id: SHL-18
# unload deactivates and removes nvm, its helpers and its variables.
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:unload nvm_echo nvm_hash_reset nvm_strip_path
nvm unload
echo "rc=$?"
command -v nvm || echo "nvm gone"
command -v nvm_find_up || echo "nvm_find_up gone"
echo "NVM_DIR=${NVM_DIR-unset} NVM_CD_FLAGS=${NVM_CD_FLAGS-unset} NVM_BIN=${NVM_BIN-unset}"
command -v node || echo "no node"
