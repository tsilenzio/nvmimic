# id: INS-12b
# Already installed runs default packages unless --skip-default-packages.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# setup: printf 'typescript\nprettier\n' > "$NVM_DIR/default-packages"
nvm install 24.15.0; echo "rc=$?"
nvm install 20.20.2 --skip-default-packages; echo "rc=$?"
