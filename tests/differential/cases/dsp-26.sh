# id: DSP-26
# unalias argument count, validation and messages.
# setup: nvm alias myalias 20.20.2
# covers: nvm nvm:unalias nvm_alias nvm_alias_path nvm_echo nvm_err nvm_iojs_prefix nvm_is_zsh
# covers: nvm_node_prefix nvm_normalize_lts nvm_print_alias_file nvm_version_dir
nvm unalias
echo "rc=$?"
nvm unalias a b
echo "rc=$?"
nvm unalias sub/name
echo "rc=$?"
nvm unalias stable
echo "rc=$?"
nvm unalias missing
echo "rc=$?"
nvm unalias myalias
