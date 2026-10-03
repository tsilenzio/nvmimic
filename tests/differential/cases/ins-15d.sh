# id: INS-15d
# --lts with --default on the already-installed path.
# setup: nvm install 24.21.0
nvm install --lts --default; echo "rc=$?"
nvm alias default; echo "rc=$?"
