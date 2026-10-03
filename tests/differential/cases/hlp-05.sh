# id: HLP-05
# The commands the README fish hook runs through bass, approximated in a bash child.
# shells: bash
# setup: (nvm install 24.15.0)
# setup: (nvm install 20.20.2)
echo 20.20.2 > .nvmrc
bash -c '. "$NVM_DIR/nvm.sh" --no-use ; nvm_find_nvmrc'
bash -c '. "$NVM_DIR/nvm.sh" --no-use ; nvm version default'
bash -c '. "$NVM_DIR/nvm.sh" --no-use ; nvm version'
bash -c '. "$NVM_DIR/nvm.sh" --no-use ; nvm version "$(cat .nvmrc)"'
bash -c '. "$NVM_DIR/nvm.sh" --no-use ; nvm use 20.20.2 >/dev/null; echo "$PATH"'
