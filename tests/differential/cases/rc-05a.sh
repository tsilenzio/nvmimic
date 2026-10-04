# id: RC-05a
# An empty .nvmrc is invalid.
# setup: nvm install 22.20.0
# setup: : > .nvmrc
# covers: nvm nvm:use nvm:which nvm_echo nvm_err nvm_find_nvmrc nvm_find_up nvm_grep nvm_has
# covers: nvm_has_colors nvm_nvmrc_invalid_msg nvm_print_color_code nvm_process_nvmrc
# covers: nvm_process_nvmrc_content nvm_rc_version nvm_wrap_with_color_code
nvm use; echo "rc=$?"
nvm which
