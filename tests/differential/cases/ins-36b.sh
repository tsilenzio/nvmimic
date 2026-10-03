# id: INS-36b
# prefix or globalconfig in a builtin, global, user or project npmrc make nvm use exit 11.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
d="$NVM_DIR/versions/node/v24.15.0"
printf 'prefix=/opt/x\n' > "$d/lib/node_modules/npm/npmrc"
nvm use 24.15.0; echo "rc=$?"
rm "$d/lib/node_modules/npm/npmrc"
mkdir -p "$d/etc" && printf 'globalconfig = /opt/x\n' > "$d/etc/npmrc"
nvm use 24.15.0; echo "rc=$?"
rm "$d/etc/npmrc"
printf 'prefix=/opt/x\n' > "$HOME/.npmrc"
nvm use 24.15.0; echo "rc=$?"
rm "$HOME/.npmrc"
printf '{}\n' > package.json && printf 'globalconfig=/opt/x\n' > .npmrc
nvm use 24.15.0; echo "rc=$?"
nvm install 24.15.0; echo "rc=$?"
nvm current
