# id: RC-01c
# The nearest .nvmrc wins over one further up.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
# setup: echo 22.20.0 > .nvmrc
# setup: mkdir -p inner/deep && echo 20.20.2 > inner/.nvmrc
cd inner/deep || exit 99
nvm_find_up .nvmrc; echo "rc=$?"
nvm which
