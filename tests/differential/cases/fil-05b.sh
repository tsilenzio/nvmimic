# id: FIL-05b
# Comments and blank lines in a hand-written alias file are ignored.
# setup: nvm install 20.20.2
# setup: nvm install 24.15.0
mkdir -p "$NVM_DIR/alias"
printf '# pinned for ci\n\n20.20.2\n' > "$NVM_DIR/alias/ci"
nvm version ci
nvm use ci
nvm alias ci
