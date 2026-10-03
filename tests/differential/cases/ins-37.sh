# id: INS-37
# nvm use --delete-prefix runs npm config delete on the offending npmrc and succeeds.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
printf 'prefix=/opt/x\n' > "$HOME/.npmrc"
nvm use --delete-prefix 24.15.0; echo "rc=$?"
rm "$HOME/.npmrc"
printf '{}\n' > package.json && printf 'prefix=/opt/x\n' > .npmrc
nvm use --delete-prefix 24.15.0 --silent; echo "rc=$?"
d="$NVM_DIR/versions/node/v24.15.0"
printf 'prefix=/opt/x\n' > "$d/lib/node_modules/npm/npmrc"
nvm use 24.15.0 --delete-prefix; echo "rc=$?"
