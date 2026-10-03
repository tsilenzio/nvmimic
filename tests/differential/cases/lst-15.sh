# id: LST-15
# Listings carry no color codes when stdout is not a TTY.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
nvm ls; echo "rc=$?"
nvm ls-remote --lts; echo "rc=$?"
nvm alias
