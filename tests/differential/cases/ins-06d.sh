# id: INS-06d
# --offline installs a version that is only in the download cache.
# setup: nvm install 20.20.2
# setup: nvm install 24.15.0
# setup: nvm uninstall 20.20.2
nvm install --offline 20; echo "rc=$?"
