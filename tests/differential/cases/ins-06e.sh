# id: INS-06e
# --offline LTS request resolves through local LTS aliases.
# setup: nvm install 24.21.0
# setup: nvm install 22.20.0
nvm install --offline --lts; echo "rc=$?"
nvm install --offline lts/jod; echo "rc=$?"
