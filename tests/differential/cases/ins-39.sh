# id: INS-39
# covers: nvm nvm:current nvm:install nvm_echo nvm_err nvm_find_nvmrc nvm_find_up nvm_get_os
# covers: nvm_has_executable nvm_ls_current nvm_rc_version nvm_version
# A caller's variable named LTS reaches nvm install under dash, whose `local` keeps the caller's value.
LTS=iron
nvm install; echo "rc=$?"
nvm current
