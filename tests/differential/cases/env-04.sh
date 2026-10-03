# id: ENV-04
# NVM_RC_VERSION after use and exec from .nvmrc, in the caller and the child.
# setup: (nvm install 20.20.2)
echo 20.20.2 > .nvmrc
nvm use
echo "caller NVM_RC_VERSION=${NVM_RC_VERSION-unset}"
nvm exec sh -c 'echo "exec child NVM_RC_VERSION=${NVM_RC_VERSION-unset}"'
"$NVM_DIR/nvm-exec" sh -c 'echo "nvm-exec child NVM_RC_VERSION=${NVM_RC_VERSION-unset}"'
