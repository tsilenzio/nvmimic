# id: DSP-26
# unalias argument count, validation and messages.
# setup: nvm alias myalias 20.20.2
nvm unalias
echo "rc=$?"
nvm unalias a b
echo "rc=$?"
nvm unalias sub/name
echo "rc=$?"
nvm unalias stable
echo "rc=$?"
nvm unalias missing
echo "rc=$?"
nvm unalias myalias
