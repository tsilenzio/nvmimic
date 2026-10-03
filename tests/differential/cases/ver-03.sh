# id: VER-03
# A trailing dot behaves like the bare major.
# setup: nvm install 24.15.0
# setup: nvm install 24.21.0
nvm version 24.; echo "rc=$?"
nvm which 24.
