# id: INS-07b
# NVM_NO_SOURCE_FALLBACK=1 aborts after a failed binary install like -b.
# setup: mkdir -p "$NVM_DIR/.cache" && : > "$NVM_DIR/.cache/bin"
export NVM_NO_SOURCE_FALLBACK=1
nvm install 24.15.0 2>"$HOME/err"; echo "rc=$?"
grep -v -E '^(Downloading [a-z]+://|Computing checksum|Checksums match|Local cache found)|%$' "$HOME/err" >&2 || :
