# id: FIL-05a
# alias writes one file per alias holding the target.
# setup: nvm install 20.20.2
nvm alias foo 20.20.2
echo "rc=$?"
nvm alias team/bar 20
cat "$NVM_DIR/alias/foo"
cat "$NVM_DIR/alias/team/bar"
