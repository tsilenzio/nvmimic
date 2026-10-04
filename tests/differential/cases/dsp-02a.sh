# id: DSP-02a
# covers: nvm nvm:--version nvm:help nvm_echo nvm_echo_with_colors nvm_has nvm_has_colors
# covers: nvm_iojs_prefix nvm_node_prefix nvm_print_color_code nvm_print_color_legend
# covers: nvm_wrap_with_color_code
# -h prints help on stdout and exits 0.
nvm -h >"$HOME/help.out"
rc=$?
sed -n 2p "$HOME/help.out"
echo "rc=$rc"
