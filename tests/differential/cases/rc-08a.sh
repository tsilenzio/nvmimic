# id: RC-08a
# .nvmrc holding the selector node is honored.
# setup: nvm install 22.20.0
# setup: nvm install 24.21.0
# setup: nvm alias myalias 22.20.0
# setup: echo 'node' > .nvmrc
nvm which; echo "rc=$?"
nvm use
