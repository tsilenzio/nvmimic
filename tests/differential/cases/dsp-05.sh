# id: DSP-05
# --version and -v print 0.40.8 and ignore extra arguments.
nvm --version
echo "rc=$?"
nvm -v
echo "rc=$?"
nvm --version extra args
