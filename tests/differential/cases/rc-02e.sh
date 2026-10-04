# id: RC-02e
# Two bare version lines are invalid.
# setup: nvm install 22.20.0
# setup: printf '22.20.0\n20.20.2\n' > .nvmrc
# covers: nvm nvm:which nvm_echo nvm_err nvm_find_nvmrc nvm_find_up nvm_grep nvm_has nvm_has_colors
# covers: nvm_nvmrc_invalid_msg nvm_print_color_code nvm_process_nvmrc nvm_process_nvmrc_content
# covers: nvm_rc_version nvm_wrap_with_color_code
nvm which
