# id: SHL-12a
# use sets MANPATH to the version man dir plus a trailing colon when unset.
# setup: (nvm install 24.15.0)
unset MANPATH
nvm use 24.15.0 >/dev/null
echo "MANPATH=${MANPATH-unset}"
