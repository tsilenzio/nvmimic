# id: SHL-19a
# exec runs the command through nvm-exec with NODE_VERSION, PATH and the nvm variables set.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
nvm exec 20.20.2 sh -c 'echo "NODE_VERSION=$NODE_VERSION"; node --version; echo "$NVM_BIN"; echo "$NVM_INC"; echo "MANPATH=$MANPATH"; echo "NVM_CD_FLAGS=[$NVM_CD_FLAGS]"'
echo "rc=$?"
node --version
