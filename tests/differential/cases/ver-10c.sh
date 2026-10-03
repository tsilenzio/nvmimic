# id: VER-10c
# lts/* is N/A when the lts alias files are absent.
# setup: nvm install 24.21.0
# setup: rm -rf "$NVM_DIR/alias/lts"
nvm version 'lts/*'; echo "rc=$?"
nvm which 'lts/*'
