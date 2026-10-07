# id: HLP-06b
# bash_completion loads under zsh and registers a completion for nvm.
# shells: zsh
[ -s "$NVM_DIR/bash_completion" ] && echo "bash_completion present"
ZSH_DISABLE_COMPFIX=true
. "$NVM_DIR/bash_completion"
echo "rc=$?"
echo "registered=${+_comps[nvm]}"
