# id: DLD-04
# LTS codenames come from index.tab on each remote listing and refresh the lts aliases.
nvm ls-remote --lts=iron --no-colors; echo "rc=$?"
rm -rf "$NVM_DIR/alias/lts"
nvm ls-remote 24 >/dev/null
ls "$NVM_DIR/alias/lts"
cat "$NVM_DIR/alias/lts/*"
