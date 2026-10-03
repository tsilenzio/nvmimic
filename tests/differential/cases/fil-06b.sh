# id: FIL-06b
# A remote listing rewrites stale LTS alias files.
mkdir -p "$NVM_DIR/alias/lts"
echo v0.0.1 > "$NVM_DIR/alias/lts/iron"
echo lts/bogus > "$NVM_DIR/alias/lts/*"
nvm ls-remote --lts >/dev/null
cat "$NVM_DIR/alias/lts/iron"
cat "$NVM_DIR/alias/lts/*"
