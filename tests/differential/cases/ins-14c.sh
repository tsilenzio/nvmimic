# id: INS-14c
# An unwritable .nvmrc prints the warning, exits 3 and skips --alias.
# setup: nvm install 24.15.0
# setup: mkdir .nvmrc
nvm install --save --alias=dev 24.15.0; echo "rc=$?"
nvm alias dev; echo "rc=$?"
