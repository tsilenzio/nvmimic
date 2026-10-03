# id: RC-03b
# Invalid .nvmrc block from use and install.
# setup: nvm install 22.20.0
# setup: printf '22.20.0\n20.20.2\n' > .nvmrc
nvm use; echo "rc=$?"
nvm install
