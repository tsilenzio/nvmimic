# id: LST-04a
# ls with alias and keyword patterns.
# setup: nvm install 22.20.0
# setup: nvm install 24.21.0
# setup: nvm install 20.20.2
# setup: nvm use 20.20.2
nvm ls default; echo "rc=$?"
nvm ls node; echo "rc=$?"
nvm ls current; echo "rc=$?"
nvm ls 'lts/*'; echo "rc=$?"
nvm ls lts/jod; echo "rc=$?"
nvm ls system
