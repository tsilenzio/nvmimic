# id: VER-16a
# current is accepted as a spec while a version is active.
# setup: nvm install 24.15.0
# setup: nvm install 22.20.0
# setup: nvm use 22.20.0
nvm version current; echo "rc=$?"
nvm which current; echo "rc=$?"
nvm ls current; echo "rc=$?"
nvm exec current node --version; echo "rc=$?"
nvm run current --version; echo "rc=$?"
nvm use current
