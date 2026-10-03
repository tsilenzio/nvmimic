# id: INS-38
# Installed versions stay until nvm uninstall removes them.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# setup: nvm install 18.20.8
nvm install 22.20.0 >/dev/null 2>&1; echo "rc=$?"
nvm uninstall 18.20.8; echo "rc=$?"
nvm ls --no-colors --no-alias
