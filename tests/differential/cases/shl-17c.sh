# id: SHL-17c
# deactivate --silent prints nothing in both branches.
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm_echo nvm_hash_reset nvm_strip_path
nvm deactivate --silent
echo "rc=$?"
nvm deactivate --silent
echo "rc=$?"
