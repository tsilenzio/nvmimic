# id: ENV-03
# NVM_CD_FLAGS is exported as empty, -q under zsh.
echo "NVM_CD_FLAGS=[${NVM_CD_FLAGS-unset}]"
sh -c 'echo "child NVM_CD_FLAGS=[${NVM_CD_FLAGS-unset}]"'
