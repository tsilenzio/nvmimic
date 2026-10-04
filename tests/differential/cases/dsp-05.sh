# id: DSP-05
# covers: nvm nvm:--version nvm_echo
# --version and -v print 0.40.8 and ignore extra arguments.
nvm --version
echo "rc=$?"
nvm -v
echo "rc=$?"
nvm --version extra args
