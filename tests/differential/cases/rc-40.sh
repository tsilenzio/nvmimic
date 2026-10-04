# id: RC-40
# nvm_rc_version called on its own writes to file descriptor 3, which is closed, and the shell says so.
# setup: echo 24 > .nvmrc
# covers: nvm_echo nvm_find_nvmrc nvm_find_up nvm_grep nvm_process_nvmrc nvm_process_nvmrc_content
# covers: nvm_rc_version
nvm_rc_version; echo "rc=$?"
nvm_rc_version 3>&1; echo "rc=$?"
NVM_SILENT=1 nvm_rc_version; echo "rc=$?"
