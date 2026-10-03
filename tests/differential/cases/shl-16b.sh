# id: SHL-16b
# use system with no system node fails with 127.
# setup: nvm install 24.15.0
nvm use system
echo "rc=$?"
node --version
