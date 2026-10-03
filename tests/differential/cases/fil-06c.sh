# id: FIL-06c
# install of an LTS selector writes the lts aliases too.
nvm install --lts=jod >/dev/null 2>&1
echo "rc=$?"
ls "$NVM_DIR/alias/lts"
