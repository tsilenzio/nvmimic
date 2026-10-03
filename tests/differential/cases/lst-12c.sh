# id: LST-12c
# ls-remote rejects uppercase LTS names.
nvm ls-remote --lts=Iron; echo "rc=$?"
nvm ls-remote lts/Iron
