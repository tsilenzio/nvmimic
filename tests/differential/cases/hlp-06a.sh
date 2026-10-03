# id: HLP-06a
# bash_completion loads under bash and completes the static subcommand list.
# shells: bash
[ -s "$NVM_DIR/bash_completion" ] && echo "bash_completion present"
. "$NVM_DIR/bash_completion"
echo "rc=$?"
complete -p nvm
fn=$(complete -p nvm | sed 's/.*-F \([^ ]*\) .*/\1/')
COMP_WORDS=(nvm "")
COMP_CWORD=1
"$fn"
printf '%s\n' "${COMPREPLY[@]}"
COMP_WORDS=(nvm de)
"$fn"
printf '%s\n' "${COMPREPLY[@]}"
