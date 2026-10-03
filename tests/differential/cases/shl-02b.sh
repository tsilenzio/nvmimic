# id: SHL-02b
# Unknown source-time arguments are ignored and the default mode is use.
# setup: (nvm install 24.15.0)
. "$NVM_DIR/nvm.sh" --bogus extra
echo "rc=$?"
node --version
nvm current
