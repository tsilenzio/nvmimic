# id: RC-08f
# .nvmrc holding the selector 22 is honored.
# setup: nvm install 22.20.0
# setup: nvm install 24.21.0
# setup: nvm alias myalias 22.20.0
# setup: echo '22' > .nvmrc
nvm which; echo "rc=$?"
nvm use
