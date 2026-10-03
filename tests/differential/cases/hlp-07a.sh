# id: HLP-07a
# nvm_ls lists installed versions, optionally filtered.
# setup: (nvm install 24.15.0)
# setup: (nvm install 20.20.2)
# setup: (nvm install 18.20.8)
nvm_ls
echo "rc=$?"
nvm_ls 20
nvm_ls 99
echo "rc=$?"
