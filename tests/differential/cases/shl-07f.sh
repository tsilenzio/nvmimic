# id: SHL-07f
# A globalconfig in a project .npmrc fails use with 11.
# setup: (nvm install 24.15.0)
echo '{}' > package.json
echo 'globalconfig=/opt/elsewhere/npmrc' > .npmrc
nvm use 24.15.0
echo "rc=$?"
