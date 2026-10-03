# id: INS-17b
# NVM_INSTALL_LOCK_STALE steals a lock older than its minutes and the lock is released after install.
# setup: mkdir -p "$NVM_DIR/.cache/locks/v24.15.0" && touch -t 202001010000 "$NVM_DIR/.cache/locks/v24.15.0"
NVM_INSTALL_LOCK_STALE=1 nvm install 24.15.0 2>"$HOME/err"; echo "rc=$?"
grep -v -E '^(Downloading [a-z]+://|Computing checksum|Checksums match|Local cache found)|%$' "$HOME/err" >&2 || :
ls -A "$NVM_DIR/.cache/locks"
