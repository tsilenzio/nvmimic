# id: RC-04c
# which with no version and no .nvmrc adds usage and exits 127.
nvm which; echo "rc=$?"
nvm which --silent
