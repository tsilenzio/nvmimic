# id: INS-14b
# --save and -w on an installed version write .nvmrc and print the Wrote line.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
nvm install --save 24.15.0; echo "rc=$?"
cat .nvmrc
nvm install -w 20.20.2; echo "rc=$?"
