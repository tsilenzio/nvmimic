# id: DSP-29
# covers: nvm nvm:* nvm:--version nvm:help nvm_echo nvm_echo_with_colors nvm_has nvm_has_colors
# covers: nvm_iojs_prefix nvm_node_prefix nvm_print_color_code nvm_print_color_legend
# covers: nvm_wrap_with_color_code
# An unknown command that looks like one of nvmimic's own names gets nvm's
# help on stderr and status 127, like any other unknown command.
nvm __nvmimic foobar >"$HOME/out" 2>"$HOME/err"
echo "rc=$?"
sed -n 2p "$HOME/err"
wc -c <"$HOME/out"
nvm __nvmimic nvm_version node
echo "rc=$?"
nvm __internal-shell-handoff nvm_version node 2>/dev/null
echo "rc=$?"
