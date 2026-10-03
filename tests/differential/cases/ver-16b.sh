# id: VER-16b
# current as a spec with no active version.
# setup: nvm install 22.20.0
# setup: nvm deactivate
nvm version current; echo "rc=$?"
nvm which current; echo "rc=$?"
nvm ls current
