# id: INS-32a
# reinstall-packages and copy-packages usage, current-version and missing-system errors.
# setup: nvm install 24.15.0
nvm reinstall-packages; echo "rc=$?"
nvm copy-packages 20 18; echo "rc=$?"
nvm reinstall-packages 24.15.0; echo "rc=$?"
nvm copy-packages 24; echo "rc=$?"
nvm reinstall-packages system; echo "rc=$?"
