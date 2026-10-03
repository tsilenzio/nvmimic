# id: RC-06
# Found message only when no version argument is given.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
# setup: echo 22.20.0 > .nvmrc
nvm use; echo "rc=$?"
nvm use 20.20.2; echo "rc=$?"
nvm which; echo "rc=$?"
nvm which 22.20.0
