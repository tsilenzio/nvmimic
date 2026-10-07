# id: HLP-06a
# bash_completion loads under bash, and every subcommand nvm's own completion offers is among the ones offered.
# shells: bash
[ -s "$NVM_DIR/bash_completion" ] && echo "bash_completion present"
. "$NVM_DIR/bash_completion"
echo "rc=$?"
complete -p nvm
fn=$(complete -p nvm | sed 's/.*-F \([^ ]*\) .*/\1/')
# Prints each of the words named that the completion offered.
offered() {
  for word in "$@"; do
    printf '%s\n' "${COMPREPLY[@]}" | grep -qx -e "$word" && echo "$word"
  done
}
COMP_WORDS=(nvm "")
COMP_CWORD=1
"$fn"
offered help install uninstall use run exec alias unalias reinstall-packages \
  current list ls list-remote ls-remote install-latest-npm cache deactivate \
  unload version version-remote which
COMP_WORDS=(nvm de)
"$fn"
offered deactivate
unset -f offered
