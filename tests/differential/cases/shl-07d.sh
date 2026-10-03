# id: SHL-07d
# use with a PREFIX conflict fails with 11 and --delete-prefix does not clear PREFIX.
# setup: (nvm install 24.15.0)
PREFIX=/opt/elsewhere
export PREFIX
nvm use 24.15.0
echo "rc=$?"
nvm use --delete-prefix 24.15.0
echo "rc=$?"
