# id: SHL-16a
# use system deactivates nvm and reports the system node.
# setup: nvm install 24.15.0
mkdir -p "$HOME/system/bin"
printf '#!/bin/sh\necho v0.0.0-system\n' > "$HOME/system/bin/node"
chmod +x "$HOME/system/bin/node"
PATH="$PATH:$HOME/system/bin"
nvm use system
echo "rc=$?"
command -v node
echo "${NVM_BIN-unset}"
