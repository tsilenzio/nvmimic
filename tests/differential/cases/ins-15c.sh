# id: INS-15c
# --lts with --default aliases an empty target on a fresh install.
nvm install --lts --default 2>"$HOME/err"; echo "rc=$?"
grep -v -E '^(Downloading [a-z]+://|Computing checksum|Checksums match|Local cache found)|%$' "$HOME/err" >&2 || :
nvm alias default; echo "rc=$?"
