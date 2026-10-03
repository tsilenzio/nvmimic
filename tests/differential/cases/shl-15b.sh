# id: SHL-15b
# Without NVM_SYMLINK_CURRENT no current symlink is written.
# setup: nvm install 24.15.0
nvm use 24.15.0 >/dev/null
[ -e "$NVM_DIR/current" ] || [ -L "$NVM_DIR/current" ] || echo "no current"
