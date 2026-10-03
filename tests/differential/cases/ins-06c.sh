# id: INS-06c
# --offline not-found wording points at nvm ls, exit 3.
# setup: nvm install 24.15.0
nvm install --offline 99; echo "rc=$?"
nvm install --offline 20; echo "rc=$?"
nvm install --offline --lts=hydrogen 18; echo "rc=$?"
