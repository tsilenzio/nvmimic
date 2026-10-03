# id: DSP-02a
# -h prints help on stdout and exits 0.
nvm -h >"$HOME/help.out"
rc=$?
sed -n 2p "$HOME/help.out"
echo "rc=$rc"
