# id: DSP-02b
# help and --help as the command print help and exit 0.
nvm help >"$HOME/help.out"
echo "rc=$?"
sed -n 2p "$HOME/help.out"
nvm --help >"$HOME/help.out"
echo "rc=$?"
sed -n 2p "$HOME/help.out"
