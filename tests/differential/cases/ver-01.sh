# id: VER-01
# Exact versions with and without the v prefix.
# setup: nvm install 24.15.0
nvm version v24.15.0; echo "rc=$?"
nvm version 24.15.0; echo "rc=$?"
nvm which 24.15.0; echo "rc=$?"
nvm version 24.21.0; echo "rc=$?"
nvm which v24.15.0
