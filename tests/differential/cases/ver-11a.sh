# id: VER-11a
# lts/<name> resolves through its alias file.
# setup: nvm install 22.20.0
nvm version lts/jod; echo "rc=$?"
nvm which lts/jod; echo "rc=$?"
nvm version lts/iron; echo "rc=$?"
nvm which lts/iron
