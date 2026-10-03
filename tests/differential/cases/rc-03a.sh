# id: RC-03a
# Invalid .nvmrc block from which.
# setup: nvm install 22.20.0
# setup: printf 'foo=bar\nbaz=qux # note\n' > .nvmrc
nvm which
