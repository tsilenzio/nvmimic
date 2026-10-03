# id: INS-17a
# A held install lock times out after NVM_INSTALL_LOCK_TIMEOUT seconds with exit 1.
# setup: mkdir -p "$NVM_DIR/.cache/locks/v24.15.0"
NVM_INSTALL_LOCK_TIMEOUT=1 nvm install 24.15.0; echo "rc=$?"
