# id: HLP-02a
# covers: nvm_echo nvm_find_nvmrc nvm_find_up
# nvm_find_nvmrc prints the path of the nearest .nvmrc.
echo 20.20.2 > .nvmrc
mkdir -p sub/deeper
cd sub/deeper
nvm_find_nvmrc
echo "rc=$?"
