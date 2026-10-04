# id: UNI-01
# covers: nvm nvm:uninstall nvm_echo nvm_err
# uninstall needs exactly one argument, else four usage lines and exit 127.
nvm uninstall
echo "rc=$?"
nvm uninstall 20.20.2 22.20.0
