# id: DLD-06a
# An empty expected checksum fails the binary download.
# setup: alias sha256sum=: shasum=: sha256=: gsha256sum=: openssl=: bssl=: sha1sum=: sha1=:
export NVM_MAKE_JOBS=1
nvm install -b --no-progress 24.15.0; echo "rc=$?"
