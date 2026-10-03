# id: UNI-07
# uninstall deletes aliases whose file contains the full version.
# setup: nvm install 20.20.2
# setup: nvm install 22.20.0
# setup: nvm alias exact v20.20.2
# setup: nvm alias major 20
nvm uninstall 20.20.2
echo "rc=$?"
nvm alias
