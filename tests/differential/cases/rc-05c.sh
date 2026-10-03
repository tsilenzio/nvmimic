# id: RC-05c
# A comment-only .nvmrc is invalid.
# setup: nvm install 22.20.0
# setup: printf '# nothing here\n' > .nvmrc
nvm use; echo "rc=$?"
nvm which
