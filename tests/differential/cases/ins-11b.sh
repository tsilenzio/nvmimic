# id: INS-11b
# A broken installed tree fails validation and is reinstalled.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
: > "$NVM_DIR/versions/node/v24.15.0/bin/node"
nvm install 24.15.0 2>"$HOME/err"; echo "rc=$?"
grep -v -E '^(Downloading [a-z]+://|Computing checksum|Checksums match|Local cache found)|%$' "$HOME/err" >&2 || :
node --version
