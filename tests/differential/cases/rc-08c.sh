# id: RC-08c
# .nvmrc holding the selector lts/* is honored.
# setup: nvm install 22.20.0
# setup: nvm install 24.21.0
# setup: nvm alias myalias 22.20.0
# setup: echo 'lts/*' > .nvmrc
nvm which; echo "rc=$?"
nvm use
