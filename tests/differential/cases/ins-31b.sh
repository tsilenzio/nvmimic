# id: INS-31b
# install-latest-npm picks the npm target by node version.
# setup: nvm install 14.21.3
# setup: nvm install 18.20.8
# setup: nvm install 20.20.2
# setup: nvm install 24.15.0
for v in 14.21.3 18.20.8 20.20.2 24.15.0; do
  nvm use "$v" >/dev/null
  nvm install-latest-npm; echo "rc=$?"
done
