# id: FIL-06a
# A remote listing writes alias/lts/* and one alias per LTS codename.
nvm ls-remote >/dev/null
echo "rc=$?"
ls "$NVM_DIR/alias/lts"
cat "$NVM_DIR/alias/lts/*"
