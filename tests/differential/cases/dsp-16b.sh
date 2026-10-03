# id: DSP-16b
# exec skips --silent and --lts=<name> before the command.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
nvm exec --silent --lts=jod node app.js
