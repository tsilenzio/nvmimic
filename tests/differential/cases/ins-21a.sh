# id: INS-21a
# -b installs from a binary when one is available.
nvm install -b 24.15.0 2>"$HOME/err"; echo "rc=$?"
grep -v -E '^(Downloading [a-z]+://|Computing checksum|Checksums match|Local cache found)|%$' "$HOME/err" >&2 || :
