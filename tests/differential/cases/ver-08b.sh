# id: VER-08b
# system with a node outside NVM_DIR on PATH.
# setup: mkdir -p "$HOME/sys" && printf '#!/bin/sh\necho v18.0.0\n' > "$HOME/sys/node" && chmod +x "$HOME/sys/node"
# setup: PATH="$HOME/sys:$PATH"; export PATH
nvm version system; echo "rc=$?"
nvm which system
