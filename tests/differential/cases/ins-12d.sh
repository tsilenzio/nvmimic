# id: INS-12d
# Already installed sets a missing default alias, then --save, then --alias.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# setup: nvm unalias default
nvm install --save --alias=dev 24.15.0; echo "rc=$?"
nvm alias dev
nvm alias default
