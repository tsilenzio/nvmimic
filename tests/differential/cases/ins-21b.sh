# id: INS-21b
# -b with a failed binary download aborts with exit 2.
# setup: mkdir -p "$NVM_DIR/.cache" && : > "$NVM_DIR/.cache/bin"
nvm install -b --no-progress 24.15.0; echo "rc=$?"
