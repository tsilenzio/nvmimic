# id: RC-02c
# A node= key is invalid.
# setup: nvm install 22.20.0
# setup: printf 'node=22.20.0\n22.20.0\n' > .nvmrc
# covers: nvm nvm:which nvm_echo nvm_err nvm_find_nvmrc nvm_find_up nvm_grep nvm_has nvm_has_colors
# covers: nvm_nvmrc_invalid_msg nvm_print_color_code nvm_process_nvmrc nvm_process_nvmrc_content
# covers: nvm_rc_version nvm_wrap_with_color_code
nvm which
