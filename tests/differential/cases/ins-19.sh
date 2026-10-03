# id: INS-19
# A fresh install switches to the new version.
# setup: nvm install 24.15.0
nvm install 20.20.2 2>"$HOME/err"; echo "rc=$?"
grep -v -E '^(Downloading [a-z]+://|Computing checksum|Checksums match|Local cache found)|%$' "$HOME/err" >&2 || :
nvm current
node --version
