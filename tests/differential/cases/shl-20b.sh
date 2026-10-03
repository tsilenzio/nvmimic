# id: SHL-20b
# exec --lts prints the latest LTS banner form.
# setup: (nvm install 24.21.0)
# setup: (nvm install 20.20.2)
nvm exec --lts node --version
echo "rc=$?"
