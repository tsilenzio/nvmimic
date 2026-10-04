# id: INS-01
# covers: nvm nvm:install nvm_echo nvm_err nvm_get_os
# Leading arguments starting with --- are rejected as typos with exit 55.
nvm install ---lts; echo "rc=$?"
nvm install --lts ---default 24; echo "rc=$?"
