# id: DSP-17c
# --silent suppresses the fallback warning.
# setup: nvm install 20.20.2
nvm exec --silent npm app.js
echo "rc=$?"
nvm run --silent app.js
