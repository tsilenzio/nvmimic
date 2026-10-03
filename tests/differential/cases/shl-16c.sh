# id: SHL-16c
# use system --silent prints nothing.
# setup: nvm install 24.15.0
mkdir -p "$HOME/system/bin"
printf '#!/bin/sh\necho v0.0.0-system\n' > "$HOME/system/bin/node"
chmod +x "$HOME/system/bin/node"
PATH="$PATH:$HOME/system/bin"
nvm use --silent system
echo "rc=$?"
