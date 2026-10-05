# id: FIL-07
# covers: nvm nvm:alias nvm_alias nvm_alias_path nvm_echo nvm_err nvm_get_colors nvm_grep nvm_has
# covers: nvm_has_colors nvm_iojs_prefix nvm_is_zsh nvm_list_aliases nvm_ls nvm_ls_current
# covers: nvm_node_prefix nvm_normalize_lts nvm_nvmrc_invalid_msg nvm_print_alias_file
# covers: nvm_print_alias_path nvm_print_color_code nvm_print_formatted_alias
# covers: nvm_process_nvmrc_content nvm_version nvm_version_dir nvm_wrap_with_color_code
# NUL bytes in an alias file come out the way the shell's `read` leaves them.
mkdir -p "$NVM_DIR/alias"
printf 'v22.1.0\000garbage\n\000\nab\000\000cd#x\n \000 \nz\000 \n' > "$NVM_DIR/alias/nul"
nvm_alias nul | tr '\000' '@'
nvm_print_alias_file "$NVM_DIR/alias/nul" | tr '\000' '@'
nvm alias nul 2>&1 | tr '\000' '@'
