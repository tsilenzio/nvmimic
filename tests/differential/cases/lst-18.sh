# id: LST-18
# which prints the installed node binary path.
# setup: nvm install 22.20.0
# setup: nvm install 24.15.0
nvm which 22; echo "rc=$?"
nvm which default; echo "rc=$?"
nvm which v24.15.0
