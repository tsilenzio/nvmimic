# id: SHL-12b
# use prepends to an existing MANPATH and appends the trailing colon.
# setup: (nvm install 24.15.0)
MANPATH=/usr/share/man
export MANPATH
nvm use 24.15.0 >/dev/null
echo "MANPATH=$MANPATH"
