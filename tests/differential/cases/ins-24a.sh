# id: INS-24a
# --latest-npm on a fresh install runs install-latest-npm.
nvm install --latest-npm 24.15.0 2>"$HOME/err"; echo "rc=$?"
grep -v -E '^(Downloading [a-z]+://|Computing checksum|Checksums match|Local cache found)|%$' "$HOME/err" >&2 || :
