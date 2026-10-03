# id: HLP-08
# Completion after alias and unalias offers names from $NVM_DIR/alias.
# shells: bash
# setup: (nvm install 20.20.2)
# setup: nvm alias myalias 20.20.2
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
