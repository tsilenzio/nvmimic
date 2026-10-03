# id: INS-12c
# Already installed with --reinstall-packages-from runs reinstall-packages.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
nvm install --reinstall-packages-from=20 24.15.0; echo "rc=$?"
