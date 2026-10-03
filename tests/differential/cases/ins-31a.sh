# id: INS-31a
# install-latest-npm rejects arguments with exit 127 and needs a node version.
nvm install-latest-npm foo; echo "rc=$?"
nvm install-latest-npm; echo "rc=$?"
