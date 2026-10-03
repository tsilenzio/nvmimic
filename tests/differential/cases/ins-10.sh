# id: INS-10
# Version-not-found messages for plain, LTS-filtered and named LTS requests, exit 3.
nvm install 99; echo "rc=$?"
nvm install v24.99.1; echo "rc=$?"
nvm install --lts 99; echo "rc=$?"
nvm install --lts=nope; echo "rc=$?"
nvm install lts/nope; echo "rc=$?"
nvm install --lts=iron 99; echo "rc=$?"
