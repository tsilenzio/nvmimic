# id: INS-31a
# covers: nvm nvm:install-latest-npm nvm_echo nvm_err nvm_install_latest_npm nvm_iojs_prefix
# covers: nvm_ls_current nvm_strip_iojs_prefix
# install-latest-npm rejects arguments with exit 127 and needs a node version.
nvm install-latest-npm foo; echo "rc=$?"
nvm install-latest-npm; echo "rc=$?"
