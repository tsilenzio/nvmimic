# id: SHL-12c
# use keeps an existing empty MANPATH entry and replaces the nvm entry on switch.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
MANPATH=/usr/share/man::/opt/man
export MANPATH
nvm use 24.15.0 >/dev/null
echo "MANPATH=$MANPATH"
nvm use 20.20.2 >/dev/null
echo "MANPATH=$MANPATH"
