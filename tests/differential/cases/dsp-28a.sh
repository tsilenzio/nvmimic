# id: DSP-28a
# version-remote LTS filters and their combination with a pattern.
nvm version-remote --lts
echo "rc=$?"
nvm version-remote --lts=iron
echo "rc=$?"
nvm version-remote lts/krypton
echo "rc=$?"
nvm version-remote --lts 24.15
echo "rc=$?"
nvm version-remote 22 --lts=krypton
