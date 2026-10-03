# id: LST-16a
# set-colors exports NVM_COLORS and reports the new colors.
nvm set-colors rgBcm; echo "rc=$?"
printf 'NVM_COLORS=%s\n' "$NVM_COLORS"
