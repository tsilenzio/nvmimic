# id: FIL-02
# Installs appear under $NVM_DIR/versions/node/vX.Y.Z with a runnable bin/node.
nvm install 22.20.0 >/dev/null 2>&1
ls "$NVM_DIR/versions/node"
nvm which 22.20.0
"$NVM_DIR/versions/node/v22.20.0/bin/node" --version
[ -d "$NVM_DIR/versions/node/v22.20.0/include/node" ] && echo "include present"
echo "$NVM_BIN"
