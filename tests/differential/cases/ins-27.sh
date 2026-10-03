# id: INS-27
# Trailing arguments on a source build print the additional options line.
export NVM_MAKE_JOBS=1
nvm install -s --no-progress 24.15.0 --with-intl=none --without-npm; echo "rc=$?"
