# id: FIL-05c
# Alias names with a .. component are rejected on write and read.
# setup: nvm install 20.20.2
nvm alias ../escape 20.20.2
echo "rc=$?"
nvm alias a/../../escape 20.20.2
echo "rc=$?"
nvm use ../escape
echo "rc=$?"
ls "$NVM_DIR"
