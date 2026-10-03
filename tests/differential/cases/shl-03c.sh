# id: SHL-03c
# A default alias resolving to N/A activates nothing and ignores .nvmrc.
# setup: (nvm install 20.20.2 && nvm alias default 18.20.8)
# setup: echo 20.20.2 > .nvmrc
. "$NVM_DIR/nvm.sh"
echo "rc=$?"
command -v node || echo "no node"
