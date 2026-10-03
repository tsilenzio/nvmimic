# id: HLP-07b
# Completion after use offers installed versions and aliases.
# shells: bash
# setup: (nvm install 24.15.0)
# setup: (nvm install 20.20.2)
. "$NVM_DIR/bash_completion"
fn=$(complete -p nvm | sed 's/.*-F \([^ ]*\) .*/\1/')
COMP_WORDS=(nvm use "")
COMP_CWORD=2
"$fn"
printf '%s\n' "${COMPREPLY[@]}" | sort
COMP_WORDS=(nvm use v2)
"$fn"
printf '%s\n' "${COMPREPLY[@]}" | sort
