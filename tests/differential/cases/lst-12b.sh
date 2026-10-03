# id: LST-12b
# ls-remote filtered to one LTS line by name.
nvm ls-remote --lts=krypton; echo "rc=$?"
nvm ls-remote lts/iron; echo "rc=$?"
nvm ls-remote --lts=nosuch
