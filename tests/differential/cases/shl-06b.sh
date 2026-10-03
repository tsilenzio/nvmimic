# id: SHL-06b
# A failing auto use aborts a caller running under set -e.
# setup: (nvm install 24.15.0)
PREFIX=/opt/elsewhere
export PREFIX
set -e
. "$NVM_DIR/nvm.sh" 2>/dev/null
echo "not reached"
