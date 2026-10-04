# id: INS-06b
# covers: nvm nvm:install nvm_alias nvm_alias_path nvm_echo nvm_err nvm_get_os nvm_iojs_prefix
# covers: nvm_node_prefix nvm_normalize_lts nvm_resolve_alias nvm_validate_implicit_alias
# covers: nvm_version_dir
# --offline LTS requests need local LTS aliases, else exit 3.
nvm install --offline --lts; echo "rc=$?"
nvm install --offline --lts=iron; echo "rc=$?"
nvm install --offline lts/jod; echo "rc=$?"
