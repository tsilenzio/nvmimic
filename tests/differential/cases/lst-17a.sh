# id: LST-17a
# covers: nvm nvm:current nvm_echo nvm_ls_current nvm_version
# current prints none with nothing active and ignores arguments.
nvm current; echo "rc=$?"
nvm current extra args
