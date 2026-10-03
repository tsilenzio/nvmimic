# id: RC-02d
# Duplicate keys are invalid.
# setup: nvm install 22.20.0
# setup: printf 'a=1\na=2\n22.20.0\n' > .nvmrc
nvm which
