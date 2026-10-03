# id: INS-11a
# An installed version prints the already-installed line, then nvm use output.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
nvm install 24.15.0; echo "rc=$?"
nvm install 20; echo "rc=$?"
nvm install v24.15.0; echo "rc=$?"
nvm current
