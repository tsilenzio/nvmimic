# id: RC-08e
# .nvmrc holding the selector myalias is honored.
# setup: nvm install 22.20.0
# setup: nvm install 24.21.0
# setup: nvm alias myalias 22.20.0
# setup: echo 'myalias' > .nvmrc
nvm which; echo "rc=$?"
nvm use
