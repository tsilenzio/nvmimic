# id: LST-02a
# ls appends a system line when a system node exists.
# setup: mkdir -p "$HOME/sys" && printf '#!/bin/sh\necho v18.0.0\n' > "$HOME/sys/node" && chmod +x "$HOME/sys/node"
# setup: PATH="$HOME/sys:$PATH"; export PATH
# setup: nvm install 22.20.0
nvm ls --no-alias
