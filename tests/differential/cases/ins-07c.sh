# id: INS-07c
# NVM_NO_SOURCE_FALLBACK with a value other than 1 still falls back to source.
# setup: mkdir -p "$NVM_DIR/.cache" && : > "$NVM_DIR/.cache/bin"
export NVM_NO_SOURCE_FALLBACK=true NVM_MAKE_JOBS=1
nvm install --no-progress 24.15.0; echo "rc=$?"
