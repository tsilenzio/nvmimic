# id: DLD-08a
# Reinstalling reuses the cached archive under .cache/bin.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# setup: nvm uninstall 24.15.0
nvm install --no-progress 24.15.0; echo "rc=$?"
ls "$NVM_DIR/.cache/bin" "$NVM_DIR/.cache/bin/node-v24.15.0-darwin-arm64"
