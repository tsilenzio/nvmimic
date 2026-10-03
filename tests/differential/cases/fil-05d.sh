# id: FIL-05d
# A missing alias resolves to N/A and fails use.
# setup: nvm install 20.20.2
nvm version nope
echo "rc=$?"
nvm use nope
echo "rc=$?"
nvm alias nope
echo "rc=$?"
