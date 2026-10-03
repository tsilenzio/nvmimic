# id: INS-14a
# --save on a fresh install does not write .nvmrc.
nvm install --save 24.15.0 2>"$HOME/err"; echo "rc=$?"
grep -v -E '^(Downloading [a-z]+://|Computing checksum|Checksums match|Local cache found)|%$' "$HOME/err" >&2 || :
ls -A
