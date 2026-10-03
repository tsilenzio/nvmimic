# id: LST-12a
# ls-remote --lts and lts/* list every LTS release.
# setup: nvm install 22.20.0
nvm ls-remote --lts; echo "rc=$?"
nvm ls-remote 'lts/*'
