# id: INS-01
# Leading arguments starting with --- are rejected as typos with exit 55.
nvm install ---lts; echo "rc=$?"
nvm install --lts ---default 24; echo "rc=$?"
