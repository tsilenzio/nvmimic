# id: DSP-02c
# covers: nvm nvm:--version nvm:help nvm_echo nvm_echo_with_colors nvm_has nvm_has_colors
# covers: nvm_iojs_prefix nvm_node_prefix nvm_print_color_code nvm_print_color_legend
# covers: nvm_wrap_with_color_code
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
