# id: VER-04a
# node resolves to the highest installed version.
# setup: nvm install 22.20.0
# setup: nvm install 24.15.0
# setup: nvm use 22.20.0
nvm version node; echo "rc=$?"
nvm which node
