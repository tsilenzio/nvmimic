# id: DSP-02c
# A help token in a later position wins over the subcommand.
nvm install 20.20.2 -h >"$HOME/help.out"
echo "rc=$?"
sed -n 2p "$HOME/help.out"
nvm exec 20.20.2 node --help >"$HOME/help.out"
echo "rc=$?"
sed -n 2p "$HOME/help.out"
nvm alias help >"$HOME/help.out"
echo "rc=$?"
sed -n 2p "$HOME/help.out"
