# id: SHL-03a
# Sourcing activates the default alias silently.
# setup: (nvm install 24.15.0)
# setup: (nvm install 20.20.2)
. "$NVM_DIR/nvm.sh"
echo "rc=$?"
node --version
nvm current
