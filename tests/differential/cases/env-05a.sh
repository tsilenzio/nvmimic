# id: ENV-05a
# covers: nvm nvm:set-colors nvm_echo nvm_grep nvm_has nvm_has_colors nvm_set_colors
# set-colors exports NVM_COLORS.
nvm set-colors rgbcy
echo "rc=$?"
echo "NVM_COLORS=${NVM_COLORS-unset}"
sh -c 'echo "child NVM_COLORS=${NVM_COLORS-unset}"'
