# id: SHL-30a
# A caller that replaces one of nvm's functions gets its own called by nvm, and later calls still work.
# shells: bash sh zsh
# setup: nvm install 24.15.0
# covers: nvm nvm:current nvm:version nvm_echo nvm_ls_current nvm_node_prefix nvm_tree_contains_path
# covers: nvm_version nvm_version_dir
nvm_ls() { echo "v9.9.9"; echo "my nvm_ls got: $*" >&2; }
nvm_version 22; echo "rc=$?"
nvm version 24; echo "rc=$?"
unset -f nvm_ls
nvm current; echo "rc=$?"
