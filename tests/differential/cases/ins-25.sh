# id: INS-25
# -s skips the binary and builds from source, which fails without a source tarball.
export NVM_MAKE_JOBS=1
nvm install -s --no-progress 24.15.0; echo "rc=$?"
nvm ls 24.15.0; echo "rc=$?"
