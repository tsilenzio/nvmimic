# id: VER-14
# Custom alias names can be set, listed and used as specs.
# setup: nvm install 22.20.0
# setup: nvm install 24.15.0
nvm alias myapp 22; echo "rc=$?"
nvm alias myapp; echo "rc=$?"
nvm version myapp; echo "rc=$?"
nvm which myapp; echo "rc=$?"
nvm use myapp
