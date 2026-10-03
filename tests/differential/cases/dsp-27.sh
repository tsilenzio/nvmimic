# id: DSP-27
# version reads only its first argument, empty means current, flags give N/A.
# setup: nvm install 20.20.2
nvm version
echo "rc=$?"
nvm version ""
echo "rc=$?"
nvm version 20 22
echo "rc=$?"
nvm version --lts
