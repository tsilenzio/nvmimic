# id: DSP-25
# alias name validation and the missing-target warning.
nvm alias 'bad#name' 20.20.2
echo "rc=$?"
nvm alias 'bad#name'
echo "rc=$?"
nvm alias sub/name 20.20.2
echo "rc=$?"
nvm alias ghost 99.0.0
