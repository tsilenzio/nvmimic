# id: ENV-02b
# Several trailing slashes are all stripped with one warning.
script="$NVM_DIR/nvm.sh"
NVM_DIR="$NVM_DIR///"
. "$script"
echo "NVM_DIR=$NVM_DIR"
