# id: SHL-17c
# deactivate --silent prints nothing in both branches.
# setup: nvm install 24.15.0
nvm deactivate --silent
echo "rc=$?"
nvm deactivate --silent
echo "rc=$?"
