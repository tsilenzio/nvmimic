# id: INS-07a
# covers: nvm nvm:install nvm_echo nvm_err nvm_get_os
# NVM_NO_SOURCE_FALLBACK=1 with -s exits 6, other values are ignored.
export NVM_NO_SOURCE_FALLBACK=1
nvm install -s 24.15.0; echo "rc=$?"
nvm install -s -b 24.15.0; echo "rc=$?"
