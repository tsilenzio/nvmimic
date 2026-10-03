# id: UNI-09a
# uninstall removes the version directory under versions/node.
# setup: nvm install 20.20.2
# setup: nvm install 22.20.0
nvm uninstall 20 >/dev/null 2>&1
echo "rc=$?"
ls "$NVM_DIR/versions/node"
