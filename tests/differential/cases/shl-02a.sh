# id: SHL-02a
# Sourcing with --no-use activates nothing even with a default alias.
# setup: (nvm install 24.15.0)
# covers: nvm nvm:current nvm_auto nvm_echo nvm_is_zsh nvm_ls_current nvm_process_parameters
# covers: nvm_version
. "$NVM_DIR/nvm.sh" --no-use
echo "rc=$?"
command -v node || echo "no node"
nvm current
