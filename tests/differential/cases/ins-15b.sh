# id: INS-15b
# --alias and --default on the already-installed path, exit code propagated.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
nvm install --alias=dev 24.15.0; echo "rc=$?"
nvm install --default 20; echo "rc=$?"
nvm install --alias=node 24.15.0; echo "rc=$?"
nvm install --alias=a#b 24.15.0; echo "rc=$?"
nvm install --alias=x/y 24.15.0; echo "rc=$?"
nvm alias default
