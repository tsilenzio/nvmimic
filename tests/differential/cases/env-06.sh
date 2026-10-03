# id: ENV-06
# Listing output without a TTY has no color codes for any TERM.
# setup: nvm install 24.15.0
TERM=dumb nvm ls 24
TERM=xterm-256color nvm ls 24
