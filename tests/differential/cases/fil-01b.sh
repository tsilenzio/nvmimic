# id: FIL-01b
# nvm-exec without NODE_VERSION uses .nvmrc.
# setup: (nvm install 24.15.0)
# setup: (nvm install 20.20.2)
echo 20.20.2 > .nvmrc
"$NVM_DIR/nvm-exec" node --version
echo "rc=$?"
