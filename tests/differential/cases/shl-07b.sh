# id: SHL-07b
# A prefix in ~/.npmrc is reported on stderr while sourcing.
# setup: (nvm install 24.15.0)
echo 'prefix=/opt/elsewhere' > "$HOME/.npmrc"
. "$NVM_DIR/nvm.sh"
echo "rc=$?"
