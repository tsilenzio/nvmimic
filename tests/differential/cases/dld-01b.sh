# id: DLD-01b
# install with an invalid mirror fails resolution.
NVM_NODEJS_ORG_MIRROR='https://example.com/a b' nvm install 24.15.0; echo "rc=$?"
NVM_NODEJS_ORG_MIRROR='file:///tmp/dist' nvm install 24.15.0; echo "rc=$?"
