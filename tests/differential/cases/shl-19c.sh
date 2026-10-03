# id: SHL-19c
# exec passes the command's own exit status through.
# setup: (nvm install 20.20.2)
nvm exec 20.20.2 sh -c 'exit 7'
echo "rc=$?"
