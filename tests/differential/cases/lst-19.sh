# id: LST-19
# which not-installed message for plain and alias specs.
# setup: nvm install 24.15.0
# setup: nvm alias old 18.20.8
nvm which 20.20.2; echo "rc=$?"
nvm which old; echo "rc=$?"
nvm which lts/iron; echo "rc=$?"
nvm which 'lts/*'
