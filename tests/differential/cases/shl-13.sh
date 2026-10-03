# id: SHL-13
# use exports NVM_BIN and NVM_INC for the selected version.
# setup: (nvm install 24.15.0)
nvm use 24.15.0 >/dev/null
echo "$NVM_BIN"
echo "$NVM_INC"
sh -c 'echo "child: $NVM_BIN $NVM_INC"'
