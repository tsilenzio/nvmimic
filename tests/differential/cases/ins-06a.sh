# id: INS-06a
# --offline resolves a partial version from installed versions.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
nvm install --offline 24; echo "rc=$?"
