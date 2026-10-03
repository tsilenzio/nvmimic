# id: LST-09
# Alias line shapes for collapsed, expanded and uninstalled targets.
# setup: nvm install 24.15.0
# setup: nvm alias exact v24.15.0
# setup: nvm alias novee 24.15.0
# setup: nvm alias missing 20.20.2
nvm alias exact; echo "rc=$?"
nvm alias novee; echo "rc=$?"
nvm alias missing
