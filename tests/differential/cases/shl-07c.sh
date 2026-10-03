# id: SHL-07c
# A lowercase npm_config_prefix outside NVM_DIR is reported while sourcing.
# setup: (nvm install 24.15.0)
npm_config_prefix=/opt/elsewhere
export npm_config_prefix
. "$NVM_DIR/nvm.sh"
echo "rc=$?"
