# id: INS-13
# Only the first install creates the default alias.
nvm install 20 2>"$HOME/err"; echo "rc=$?"
grep -v -E '^(Downloading [a-z]+://|Computing checksum|Checksums match|Local cache found)|%$' "$HOME/err" >&2 || :
nvm install 24.15.0 2>"$HOME/err"; echo "rc=$?"
grep -v -E '^(Downloading [a-z]+://|Computing checksum|Checksums match|Local cache found)|%$' "$HOME/err" >&2 || :
nvm alias default
