# id: RC-08d
# .nvmrc holding the selector lts/-1 is honored.
# setup: nvm install 22.20.0
# setup: nvm install 24.21.0
# setup: nvm alias myalias 22.20.0
# setup: echo 'lts/-1' > .nvmrc
nvm which; echo "rc=$?"
nvm use
