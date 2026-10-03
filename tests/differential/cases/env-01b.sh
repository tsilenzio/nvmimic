# id: ENV-01b
# A set NVM_DIR is kept and exported.
echo "NVM_DIR=$NVM_DIR"
sh -c 'echo "child NVM_DIR=${NVM_DIR-unset}"'
