# id: RC-03b
# Invalid .nvmrc block from use and install.
# setup: nvm install 22.20.0
# setup: printf '22.20.0\n20.20.2\n' > .nvmrc
# covers: nvm nvm:install nvm:use nvm_echo nvm_err nvm_find_nvmrc nvm_find_up nvm_get_os nvm_grep
# covers: nvm_has nvm_has_colors nvm_has_executable nvm_nvmrc_invalid_msg nvm_print_color_code
# covers: nvm_process_nvmrc nvm_process_nvmrc_content nvm_rc_version nvm_wrap_with_color_code
nvm use; echo "rc=$?"
nvm install
