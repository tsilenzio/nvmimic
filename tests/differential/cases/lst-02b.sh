# id: LST-02b
# ls marks system as current after nvm use system.
# setup: mkdir -p "$HOME/sys" && printf '#!/bin/sh\necho v18.0.0\n' > "$HOME/sys/node" && chmod +x "$HOME/sys/node"
# setup: PATH="$HOME/sys:$PATH"; export PATH
# setup: nvm install 22.20.0
# setup: nvm use system
nvm ls --no-alias; echo "rc=$?"
nvm current
