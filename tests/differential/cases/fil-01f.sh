# id: FIL-01f
# nvm-exec with an .nvmrc naming an uninstalled version reports it and fails with 127.
# setup: (nvm install 24.15.0)
echo 18.20.8 > .nvmrc
"$NVM_DIR/nvm-exec" node --version
echo "rc=$?"
