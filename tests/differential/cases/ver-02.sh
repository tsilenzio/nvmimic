# id: VER-02
# Partial versions pick the highest installed match on a dot boundary.
# setup: nvm install 24.15.0
# setup: nvm install 24.21.0
# setup: nvm install 22.20.0
nvm version 24; echo "rc=$?"
nvm version 24.15; echo "rc=$?"
nvm version v22; echo "rc=$?"
nvm version 2; echo "rc=$?"
nvm which 24
