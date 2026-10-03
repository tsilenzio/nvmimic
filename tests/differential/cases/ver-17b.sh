# id: VER-17b
# An alias cycle resolves to infinity and which aborts with exit 8.
# setup: nvm install 22.20.0
# setup: nvm alias loopa loopb
# setup: nvm alias loopb loopa
nvm alias loopa; echo "rc=$?"
nvm version loopa; echo "rc=$?"
nvm which loopa
