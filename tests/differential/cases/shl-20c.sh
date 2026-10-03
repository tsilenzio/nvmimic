# id: SHL-20c
# exec --lts=<name> prints the named LTS banner form.
# setup: (nvm install 24.21.0)
# setup: (nvm install 20.20.2)
nvm exec --lts=iron node --version
echo "rc=$?"
