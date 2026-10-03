# id: LST-07a
# Implicit alias lines carry the default suffix.
# setup: nvm install 24.15.0
nvm alias iojs; echo "rc=$?"
nvm alias node; echo "rc=$?"
nvm alias stable; echo "rc=$?"
nvm alias unstable
