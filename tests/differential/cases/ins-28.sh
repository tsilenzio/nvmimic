# id: INS-28
# A failed binary install falls back to a source build.
# setup: mkdir -p "$NVM_DIR/.cache" && : > "$NVM_DIR/.cache/bin"
export NVM_MAKE_JOBS=1
nvm install --no-progress 24.15.0; echo "rc=$?"
