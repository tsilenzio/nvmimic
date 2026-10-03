# id: LST-10a
# alias with a prefix lists matches across all groups.
# setup: nvm install 22.20.0
# setup: nvm alias site 22
nvm alias s; echo "rc=$?"
nvm alias d; echo "rc=$?"
nvm alias lts/j
