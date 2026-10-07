# id: SHL-32
# A placeholder `nvm` and `node` that load nvm.sh on first use, from inside a function, the way lazy-loading setups do.
# shells: bash zsh
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:unload nvm_echo nvm_hash_reset nvm_strip_path
nvm unload
lazy() {
  nvm() { unset -f nvm node; . "$NVM_DIR/nvm.sh"; nvm "$@"; }
  node() { unset -f nvm node; . "$NVM_DIR/nvm.sh"; node "$@"; }
}
lazy
type nvm_find_nvmrc >/dev/null 2>&1; echo "loaded before=$?"
nvm current; echo "rc=$?"
type nvm_find_nvmrc >/dev/null 2>&1; echo "loaded after=$?"
nvm unload
lazy
node --version; echo "rc=$?"
nvm use 24.15.0; echo "rc=$?"
unset -f lazy
