# id: SHL-19b
# exec fails when the prefix checks inside nvm-exec fail.
# setup: (nvm install 20.20.2)
PREFIX=/opt/elsewhere
export PREFIX
nvm exec 20.20.2 node --version
echo "rc=$?"
