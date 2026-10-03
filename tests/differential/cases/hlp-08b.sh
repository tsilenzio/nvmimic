# id: HLP-08b
# Alias completion lists nested LTS alias files in the order nvm's find walks them.
# shells: bash
# covers: nvm:alias
# setup: nvm install 20.20.2
# setup: nvm ls-remote >/dev/null
# setup: nvm alias zeta 20.20.2
# setup: nvm alias alpha 20.20.2
. "$NVM_DIR/bash_completion"
COMP_WORDS=(nvm unalias "")
COMP_CWORD=2
__nvm
printf '%s\n' "${COMPREPLY[@]}"
