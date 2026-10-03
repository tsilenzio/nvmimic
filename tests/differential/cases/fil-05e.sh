# id: FIL-05e
# An unreadable alias file is reported.
# setup: nvm install 20.20.2
nvm alias locked 20.20.2 >/dev/null
chmod 000 "$NVM_DIR/alias/locked"
nvm version locked
echo "rc=$?"
nvm use locked
echo "rc=$?"
nvm alias locked
echo "rc=$?"
chmod 644 "$NVM_DIR/alias/locked"
