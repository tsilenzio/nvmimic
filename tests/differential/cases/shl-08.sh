# id: SHL-08
# Sourcing and calling nvm leave the caller's zsh options unchanged.
# shells: zsh
# setup: (nvm install 24.15.0)
before=$(setopt)
. "$NVM_DIR/nvm.sh"
nvm use 20.20.2 >/dev/null 2>&1
nvm ls >/dev/null
nvm current >/dev/null
after=$(setopt)
[ "$before" = "$after" ] && echo "options unchanged"
setopt
