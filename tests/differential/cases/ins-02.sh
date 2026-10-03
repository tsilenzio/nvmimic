# id: INS-02
# -s and -b together exit 6 in either order.
nvm install -s -b 24.15.0; echo "rc=$?"
nvm install -b -s 24.15.0; echo "rc=$?"
