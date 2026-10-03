# id: ENV-02a
# A trailing slash on NVM_DIR is stripped with a warning on stderr.
NVM_DIR="$NVM_DIR/"
. "${NVM_DIR}nvm.sh"
echo "rc=$?"
echo "NVM_DIR=$NVM_DIR"
