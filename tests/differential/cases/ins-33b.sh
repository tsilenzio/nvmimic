# id: INS-33b
# A default-packages line with whitespace fails with the one-package-per-line error.
# setup: nvm install 20.20.2
# setup: printf 'typescript\nprettier eslint\n' > "$NVM_DIR/default-packages"
nvm install 24.15.0 2>"$HOME/err"; echo "rc=$?"
grep -v -E '^(Downloading [a-z]+://|Computing checksum|Checksums match|Local cache found)|%$' "$HOME/err" >&2 || :
nvm install 20.20.2; echo "rc=$?"
printf 'typescript # inline\n' > "$NVM_DIR/default-packages"
nvm install 20.20.2; echo "rc=$?"
