# id: SHL-14
# A hashed node is forgotten after use switches versions.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
nvm use 24.15.0 >/dev/null
node --version
node --version
nvm use 20.20.2 >/dev/null
node --version
