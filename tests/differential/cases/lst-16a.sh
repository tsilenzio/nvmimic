# id: LST-16a
# covers: nvm nvm:set-colors nvm_echo nvm_grep nvm_has nvm_has_colors nvm_set_colors
# set-colors exports NVM_COLORS and reports the new colors.
nvm set-colors rgBcm; echo "rc=$?"
printf 'NVM_COLORS=%s\n' "$NVM_COLORS"
