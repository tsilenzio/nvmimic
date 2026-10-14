# nvm stand-in from tsilenzio/tap, the Homebrew formula named nvm.
#
# It sits where Homebrew's own nvm keeps nvm.sh, so a startup file that
# loads Homebrew's nvm keeps working after that nvm is gone. It loads
# whatever nvm is in $NVM_DIR: nvmimic, nvmise or official nvm.
#
# MIT licensed: Copyright (c) 2026 Taylor Silenzio.

if [ -z "${NVM_DIR-}" ]; then
  export NVM_DIR="$HOME/.nvm"
fi
# dash gives a sourced file no arguments of its own, so there nvm.sh sees
# the caller's.
# shellcheck disable=SC1091,SC2240
if [ -s "$NVM_DIR/nvm.sh" ]; then
  \. "$NVM_DIR/nvm.sh" ${1+"$@"}
else
  command printf 'nvm: there is no nvm.sh in %s, so nvm was not loaded.\n' "$NVM_DIR" >&2
  false
fi
