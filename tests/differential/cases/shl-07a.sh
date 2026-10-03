# id: SHL-07a
# A PREFIX conflict is reported on stderr while sourcing.
# setup: (nvm install 24.15.0)
PREFIX=/opt/elsewhere
export PREFIX
. "$NVM_DIR/nvm.sh"
echo "rc=$?"
