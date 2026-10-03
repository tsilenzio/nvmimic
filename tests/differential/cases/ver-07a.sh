# id: VER-07a
# latest is not a keyword without a user alias.
# setup: nvm install 24.15.0
nvm version latest; echo "rc=$?"
nvm version-remote latest; echo "rc=$?"
nvm which latest
