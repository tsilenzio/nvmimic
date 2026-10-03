# id: DSP-19
# ls flag handling and its exit 55 errors.
# setup: nvm install 20.20.2
nvm ls --foo
echo "rc=$?"
nvm ls --no-alias 20
echo "rc=$?"
nvm ls -- --no-colors --no-alias
