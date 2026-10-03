# id: SHL-10a
# use prepends the version bin dir when PATH has no nvm entry.
# setup: (nvm install 24.15.0)
nvm use 24.15.0 >/dev/null
echo "$PATH"
