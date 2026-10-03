# id: FIL-01c
# nvm-exec with no NODE_VERSION and no .nvmrc fails with 127 and the 0.40.8 text.
# setup: (nvm install 24.15.0)
"$NVM_DIR/nvm-exec" node --version
echo "rc=$?"
