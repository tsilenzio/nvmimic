# id: VER-11c
# An empty lts/ name is not an alias.
# setup: nvm install 22.20.0
nvm version lts/; echo "rc=$?"
nvm which lts/; echo "rc=$?"
nvm alias lts/
