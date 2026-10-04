# id: HLP-02b
# covers: nvm_echo nvm_find_nvmrc nvm_find_up
# nvm_find_nvmrc prints nothing when there is no .nvmrc.
nvm_find_nvmrc
echo "rc=$?"
