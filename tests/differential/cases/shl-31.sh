# id: SHL-31
# With `set -x` on, a script that catches nvm's output and passes it through `echo` still finds nvm's message in it.
# setup: nvm install 24.15.0
# covers: nvm nvm:deactivate nvm:use nvm_alias nvm_alias_path nvm_echo nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_grep nvm_has_system_iojs nvm_has_system_node
# covers: nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh nvm_ls nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_num_version_groups nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir
OUT=$(set -x; nvm use lts 2>&1); echo "rc=$?"
echo "$OUT" | grep -q 'is not an alias'; echo "found=$?"
OUT=$(set -x; curl() { command curl "$@"; }; nvm use lts 2>&1); echo "rc=$?"
echo "$OUT" | grep -q 'is not an alias'; echo "found=$?"
unset OUT
