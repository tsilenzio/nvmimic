# id: RC-07
# which --silent hides .nvmrc messages and an inherited NVM_SILENT is ignored.
# setup: nvm install 22.20.0
# setup: echo 22.20.0 > .nvmrc
nvm which --silent; echo "rc=$?"
NVM_SILENT=1; export NVM_SILENT
nvm which
