# id: VER-05a
# stable resolves like node.
# setup: nvm install 22.20.0
# setup: nvm install 24.15.0
nvm version stable; echo "rc=$?"
nvm which stable
