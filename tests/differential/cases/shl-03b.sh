# id: SHL-03b
# Without a default alias, sourcing uses the version in .nvmrc.
# setup: (nvm install 20.20.2 && nvm unalias default)
# setup: echo 20.20.2 > .nvmrc
. "$NVM_DIR/nvm.sh"
echo "rc=$?"
node --version
