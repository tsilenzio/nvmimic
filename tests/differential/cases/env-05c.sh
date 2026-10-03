# id: ENV-05c
# A preset NVM_COLORS does not change non-TTY listing output.
# setup: nvm install 24.15.0
NVM_COLORS=rgbcy
export NVM_COLORS
nvm ls 24
