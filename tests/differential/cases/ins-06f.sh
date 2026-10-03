# id: INS-06f
# --offline with a cached version but no cached archive reports the missing archive.
# setup: mkdir -p "$NVM_DIR/.cache/src/node-v20.20.2"
export NVM_MAKE_JOBS=1
nvm install --offline -b 20; echo "rc=$?"
nvm install --offline 20; echo "rc=$?"
