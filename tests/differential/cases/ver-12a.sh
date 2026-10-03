# id: VER-12a
# lts/-N counts back through the lts alias files.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
nvm version lts/-1; echo "rc=$?"
nvm version lts/-2; echo "rc=$?"
nvm which lts/-1; echo "rc=$?"
nvm ls-remote lts/-1
