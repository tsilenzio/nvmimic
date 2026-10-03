# id: DLD-09
# nvm cache dir, clear, a failed clear, and the usage error.
# setup: nvm install 24.15.0
nvm cache dir; echo "rc=$?"
nvm cache clear; echo "rc=$?"
ls -A "$NVM_DIR/.cache"
nvm cache; echo "rc=$?"
nvm cache list; echo "rc=$?"
nvm cache dir extra; echo "rc=$?"
chmod 555 "$NVM_DIR"
nvm cache clear; echo "rc=$?"
chmod 755 "$NVM_DIR"
