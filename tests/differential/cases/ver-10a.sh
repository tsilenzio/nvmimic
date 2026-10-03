# id: VER-10a
# lts/* resolves to the exact version recorded in the alias files.
# setup: nvm install 24.21.0
nvm version 'lts/*'; echo "rc=$?"
nvm which 'lts/*'
