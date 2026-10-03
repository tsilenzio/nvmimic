# id: SHL-21
# run delegates to exec with node and passes arguments through.
# setup: (nvm install 20.20.2)
nvm run 20.20.2 --version
echo "rc=$?"
nvm run 20.20.2 -e 'x y'
echo "rc=$?"
