# id: VER-12b
# lts/-N beyond the known LTS lines fails.
# setup: nvm install 22.20.0
nvm version lts/-5; echo "rc=$?"
nvm which lts/-9; echo "rc=$?"
nvm ls-remote lts/-9; echo "rc=$?"
nvm install lts/-9
