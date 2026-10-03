# id: INS-06b
# --offline LTS requests need local LTS aliases, else exit 3.
nvm install --offline --lts; echo "rc=$?"
nvm install --offline --lts=iron; echo "rc=$?"
nvm install --offline lts/jod; echo "rc=$?"
