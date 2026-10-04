# id: ENV-05b
# covers: nvm nvm:--version nvm:help nvm:set-colors nvm_echo nvm_echo_with_colors
# covers: nvm_err_with_colors nvm_grep nvm_has nvm_has_colors nvm_iojs_prefix nvm_node_prefix
# covers: nvm_print_color_code nvm_print_color_legend nvm_set_colors nvm_wrap_with_color_code
# set-colors with invalid codes prints help and the color error and leaves NVM_COLORS alone.
nvm set-colors xxxxx
echo "rc=$?"
echo "NVM_COLORS=${NVM_COLORS-unset}"
