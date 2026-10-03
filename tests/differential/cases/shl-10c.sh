# id: SHL-10c
# use prepends when the nvm entry sits behind /usr/bin, keeping the old entry.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
nvm use 24.15.0 >/dev/null
PATH="/usr/bin:$NVM_BIN:/bin"
nvm use 20.20.2 >/dev/null
echo "$PATH"
