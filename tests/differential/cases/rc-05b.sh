# id: RC-05b
# A whitespace-only .nvmrc is invalid.
# setup: nvm install 22.20.0
# setup: printf '  \n\t\n' > .nvmrc
nvm use; echo "rc=$?"
nvm which
