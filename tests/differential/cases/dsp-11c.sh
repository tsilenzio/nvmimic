# id: DSP-11c
# use prefers --lts over a version given in any position.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
nvm use 20.20.2 --lts=jod
echo "rc=$?"
nvm current
