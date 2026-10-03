# id: LST-05b
# ls with a pattern or --no-alias omits the alias listing.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
nvm ls 22; echo "rc=$?"
nvm ls --no-alias; echo "rc=$?"
nvm ls --no-alias 22
