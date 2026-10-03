# id: DLD-07a
# A download that fails prints download from <url> failed.
export NVM_MAKE_JOBS=1
nvm install -s --no-progress 24.15.0; echo "rc=$?"
