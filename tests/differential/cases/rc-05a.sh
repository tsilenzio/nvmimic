# id: RC-05a
# An empty .nvmrc is invalid.
# setup: nvm install 22.20.0
# setup: : > .nvmrc
nvm use; echo "rc=$?"
nvm which
