# id: DSP-28b
# version-remote default pattern, empty LTS filter and N/A results.
nvm version-remote
echo "rc=$?"
nvm version-remote --lts=
echo "rc=$?"
nvm version-remote 99
echo "rc=$?"
nvm version-remote default
echo "rc=$?"
nvm version-remote latest
echo "rc=$?"
nvm version-remote --foo
