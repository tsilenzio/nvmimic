# id: SHL-07e
# use --delete-prefix runs npm config delete on the offending user .npmrc.
# setup: (nvm install 24.15.0)
printf 'prefix=/opt/elsewhere\ncolor=false\n' > "$HOME/.npmrc"
nvm use 24.15.0
echo "rc=$?"
nvm use --delete-prefix 24.15.0
echo "rc=$?"
node --version
cat "$HOME/.npmrc"
