# id: RC-01a
# .nvmrc is found by walking up from a nested directory.
# setup: nvm install 22.20.0
# setup: echo 22.20.0 > .nvmrc
# setup: mkdir -p a/b/c
cd a/b/c || exit 99
nvm_find_nvmrc; echo "rc=$?"
nvm which
