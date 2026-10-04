# id: HLP-08
# Completion after alias and unalias offers names from $NVM_DIR/alias.
# shells: bash
# setup: (nvm install 20.20.2)
# setup: nvm alias myalias 20.20.2
# covers: nvm nvm:deactivate nvm_echo nvm_ensure_version_prefix nvm_err nvm_grep nvm_has_system_iojs
# covers: nvm_has_system_node nvm_iojs_prefix nvm_is_iojs_version nvm_is_zsh nvm_ls nvm_node_prefix
# covers: nvm_num_version_groups nvm_resolve_alias nvm_resolve_local_alias nvm_strip_iojs_prefix
# covers: nvm_strip_path nvm_version_dir
. "$NVM_DIR/bash_completion"
fn=$(complete -p nvm | sed 's/.*-F \([^ ]*\) .*/\1/')
COMP_WORDS=(nvm unalias "")
COMP_CWORD=2
"$fn"
printf '%s\n' "${COMPREPLY[@]}" | sort
COMP_WORDS=(nvm alias my "")
COMP_CWORD=3
"$fn"
printf '%s\n' "${COMPREPLY[@]}" | sort
