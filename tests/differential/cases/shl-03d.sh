# id: SHL-03d
# Re-sourcing keeps an nvm version that is already current.
# setup: (nvm install 24.15.0)
# setup: (nvm install 20.20.2)
nvm use 20.20.2 >/dev/null
. "$NVM_DIR/nvm.sh"
echo "rc=$?"
node --version
