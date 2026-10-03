# id: ENV-05a
# set-colors exports NVM_COLORS.
nvm set-colors rgbcy
echo "rc=$?"
echo "NVM_COLORS=${NVM_COLORS-unset}"
sh -c 'echo "child NVM_COLORS=${NVM_COLORS-unset}"'
