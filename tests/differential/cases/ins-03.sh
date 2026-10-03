# id: INS-03
# --default and --alias are mutually exclusive and single-use.
nvm install --default --alias=dev 24.15.0; echo "rc=$?"
nvm install --alias=dev --default 24.15.0; echo "rc=$?"
nvm install --default --default 24.15.0; echo "rc=$?"
nvm install --alias=a --alias=b 24.15.0; echo "rc=$?"
