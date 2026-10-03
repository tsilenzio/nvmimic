# id: VER-07b
# latest follows a user alias named latest.
# setup: nvm install 24.15.0
# setup: nvm install 22.20.0
# setup: nvm alias latest 22.20.0
nvm version latest; echo "rc=$?"
nvm which latest
