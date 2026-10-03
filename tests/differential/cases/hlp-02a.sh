# id: HLP-02a
# nvm_find_nvmrc prints the path of the nearest .nvmrc.
echo 20.20.2 > .nvmrc
mkdir -p sub/deeper
cd sub/deeper
nvm_find_nvmrc
echo "rc=$?"
