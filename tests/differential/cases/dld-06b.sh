# id: DLD-06b
# A cached archive with the wrong checksum prints the mismatch, is removed and downloaded again.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
# setup: nvm uninstall 24.15.0
# setup: printf 'corrupt\n' > "$NVM_DIR/.cache/bin/node-v24.15.0-darwin-arm64/node-v24.15.0-darwin-arm64.tar.xz"
nvm install --no-progress 24.15.0 2>"$HOME/err"; echo "rc=$?"
grep -v '%$' "$HOME/err" | sed 's/[0-9a-f]\{64\}'"'"' expected/<sum>'"'"' expected/' >&2
