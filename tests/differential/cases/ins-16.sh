# id: INS-16
# Fresh-install stdout sequence, stderr discarded.
nvm install 22 2>/dev/null; echo "rc=$?"
nvm install --alias=old 18.20.8 2>/dev/null; echo "rc=$?"
