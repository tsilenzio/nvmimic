# id: INS-04
# --save and -w may be given only once.
nvm install --save --save 24.15.0; echo "rc=$?"
nvm install -w --save 24.15.0; echo "rc=$?"
nvm install -w -w 24.15.0; echo "rc=$?"
