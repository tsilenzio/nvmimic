# id: INS-02
# covers: nvm nvm:install nvm_echo nvm_err nvm_get_os
# -s and -b together exit 6 in either order.
nvm install -s -b 24.15.0; echo "rc=$?"
nvm install -b -s 24.15.0; echo "rc=$?"
