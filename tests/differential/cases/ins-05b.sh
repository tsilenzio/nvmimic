# id: INS-05b
# Reinstall source equal to the target exits 4, a source not installed exits 5.
nvm install --reinstall-packages-from=24.15.0 24.15.0; echo "rc=$?"
nvm install 24.15.0 --copy-packages-from=v24.15.0; echo "rc=$?"
nvm install --reinstall-packages-from=20 24.15.0; echo "rc=$?"
nvm install 24.15.0 --copy-packages-from=18; echo "rc=$?"
