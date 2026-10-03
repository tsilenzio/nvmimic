# id: HLP-06b
# bash_completion loads under zsh through compinit and bashcompinit.
# shells: zsh
[ -s "$NVM_DIR/bash_completion" ] && echo "bash_completion present"
ZSH_DISABLE_COMPFIX=true
. "$NVM_DIR/bash_completion"
echo "rc=$?"
complete -p nvm
