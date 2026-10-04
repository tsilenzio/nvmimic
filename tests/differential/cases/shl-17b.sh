# id: SHL-17b
# covers: nvm nvm:deactivate nvm_echo nvm_err nvm_strip_path
# deactivate with nothing active reports not found on stderr and returns 0.
nvm deactivate
echo "rc=$?"
