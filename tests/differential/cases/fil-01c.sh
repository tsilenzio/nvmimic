# id: FIL-01c
# nvm-exec with no NODE_VERSION and no .nvmrc fails with 127 and the 0.40.8 text.
# setup: (nvm install 24.15.0)
# covers: nvm nvm:use nvm_auto nvm_echo nvm_err nvm_find_nvmrc nvm_find_up nvm_is_zsh
# covers: nvm_process_parameters nvm_rc_version
"$NVM_DIR/nvm-exec" node --version
echo "rc=$?"
