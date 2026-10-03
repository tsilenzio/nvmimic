# id: INS-32b
# reinstall-packages reinstalls listed globals and links linked ones.
# setup: nvm install 20.20.2
# setup: nvm install 24.15.0
# setup: mkdir "$HOME/mylink"
# setup: printf '#!/bin/sh\nif [ "$1" = list ]; then printf "%%s\\n" "/x/lib" "+-- corepack@0.29.0" "+-- npm@10.9.0" "+-- typescript@5.6.2" "+-- prettier@3.3.3" "+-- mylink@1.0.0 -> %s"; else echo "npm20: $*"; fi\n' "$HOME/mylink" > "$NVM_DIR/versions/node/v20.20.2/lib/node_modules/npm/bin/npm-cli.js"
nvm reinstall-packages 20; echo "rc=$?"
nvm copy-packages v20.20.2; echo "rc=$?"
