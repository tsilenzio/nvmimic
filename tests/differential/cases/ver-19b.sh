# id: VER-19b
# install resolves a partial remotely and fails on an unknown version.
nvm install 2 >/dev/null; echo "rc=$?"
nvm install 99; echo "rc=$?"
nvm install 20 >/dev/null 2>&1; echo "rc=$?"
nvm version 20
