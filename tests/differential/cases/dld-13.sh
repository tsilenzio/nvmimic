# id: DLD-13
# Node below 16 on darwin arm64 downloads the darwin-x64 archive.
nvm install --no-progress 14.21.3 2>&1 | grep -E '^Downloading'
nvm ls 14 --no-colors
ls "$NVM_DIR/.cache/bin"
