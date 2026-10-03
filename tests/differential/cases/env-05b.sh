# id: ENV-05b
# set-colors with invalid codes prints help and the color error and leaves NVM_COLORS alone.
nvm set-colors xxxxx
echo "rc=$?"
echo "NVM_COLORS=${NVM_COLORS-unset}"
