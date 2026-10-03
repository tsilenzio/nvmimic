# id: VER-13
# Bare lts is not an alias.
# setup: nvm install 22.20.0
nvm version lts; echo "rc=$?"
nvm which lts; echo "rc=$?"
nvm use lts
