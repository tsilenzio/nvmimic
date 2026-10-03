# id: INS-05a
# Malformed --reinstall-packages-from and --copy-packages-from exit 6.
nvm install --reinstall-packages-from=20 --reinstall-packages-from=18 24.15.0; echo "rc=$?"
nvm install --reinstall-packages-from=20 --copy-packages-from=18 24.15.0; echo "rc=$?"
nvm install --copy-packages-from=20 --reinstall-packages-from=18 24.15.0; echo "rc=$?"
nvm install --reinstall-packages-from= 24.15.0; echo "rc=$?"
nvm install --copy-packages-from= 24.15.0; echo "rc=$?"
nvm install --reinstall-packages-from 20 24.15.0; echo "rc=$?"
nvm install --copy-packages-from 20 24.15.0; echo "rc=$?"
nvm install 24.15.0 --reinstall-packages-from 20; echo "rc=$?"
nvm install 24.15.0 --reinstall-packages-from=20 --copy-packages-from=18; echo "rc=$?"
nvm install 24.15.0 --copy-packages-from=; echo "rc=$?"
