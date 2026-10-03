# id: RC-08b
# .nvmrc holding the selector stable is honored.
# setup: nvm install 22.20.0
# setup: nvm install 24.21.0
# setup: nvm alias myalias 22.20.0
# setup: echo 'stable' > .nvmrc
nvm which; echo "rc=$?"
nvm use
