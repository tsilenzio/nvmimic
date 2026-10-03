# id: ENV-01a
# With NVM_DIR unset, sourcing detects it from the script location and exports it.
script="$NVM_DIR/nvm.sh"
unset NVM_DIR
. "$script"
echo "rc=$?"
echo "NVM_DIR=${NVM_DIR-unset}"
sh -c 'echo "child NVM_DIR=${NVM_DIR-unset}"'
