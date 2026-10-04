# id: DSP-02b
# covers: nvm nvm:--version nvm:help nvm_echo nvm_echo_with_colors nvm_has nvm_has_colors
# covers: nvm_iojs_prefix nvm_node_prefix nvm_print_color_code nvm_print_color_legend
# covers: nvm_wrap_with_color_code
# help and --help as the command print help and exit 0.
nvm help >"$HOME/help.out"
echo "rc=$?"
sed -n 2p "$HOME/help.out"
nvm --help >"$HOME/help.out"
echo "rc=$?"
sed -n 2p "$HOME/help.out"
