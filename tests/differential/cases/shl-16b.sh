# id: SHL-16b
# use system with no system node fails with 127.
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:use nvm_echo nvm_err nvm_has_system_iojs nvm_has_system_node
# covers: nvm_hash_reset nvm_iojs_prefix nvm_match_version nvm_strip_path
nvm use system
echo "rc=$?"
node --version
