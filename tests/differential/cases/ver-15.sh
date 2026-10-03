# id: VER-15
# default holding a partial version follows new installs on that line.
# setup: nvm install 24.15.0
# setup: nvm alias default 24
nvm version default; echo "rc=$?"
nvm install 24.21.0 >/dev/null 2>&1
nvm version default; echo "rc=$?"
nvm alias default
