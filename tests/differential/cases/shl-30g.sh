# id: SHL-30g
# nvm unload after a function was replaced removes nvm, and sourcing again brings it back.
# shells: bash sh zsh
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:unload nvm_hash_reset nvm_strip_path
nvm_echo() { command printf 'mine: %s\n' "$*"; }
nvm unload; echo "rc=$?"
type nvm >/dev/null 2>&1; echo "nvm defined: $?"
type nvm_echo >/dev/null 2>&1; echo "nvm_echo defined: $?"
. "$NVM_DIR/nvm.sh"
nvm current; echo "rc=$?"
