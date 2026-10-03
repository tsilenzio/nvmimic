# id: SHL-02a
# Sourcing with --no-use activates nothing even with a default alias.
# setup: (nvm install 24.15.0)
. "$NVM_DIR/nvm.sh" --no-use
echo "rc=$?"
command -v node || echo "no node"
nvm current
