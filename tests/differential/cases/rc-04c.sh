# id: RC-04c
# covers: nvm nvm:which nvm_echo nvm_err nvm_find_nvmrc nvm_find_up nvm_rc_version
# which with no version and no .nvmrc adds usage and exits 127.
nvm which; echo "rc=$?"
nvm which --silent
