# id: INS-17c
# The install lock is per version, a lock on another version does not block.
# setup: mkdir -p "$NVM_DIR/.cache/locks/v20.20.2"
NVM_INSTALL_LOCK_TIMEOUT=1 nvm install 24.15.0 >/dev/null 2>&1; echo "rc=$?"
NVM_INSTALL_LOCK_TIMEOUT=0 nvm install 20.20.2; echo "rc=$?"
