# id: DLD-03
# NVM_AUTH_HEADER is sent sanitized as an Authorization header on every download.
# setup: mkdir "$HOME/wrap" && printf '#!/bin/sh\nfor a in "$@"; do case "$a" in Authorization:*) echo "$a" >> "$HOME/headers";; esac; done\nexec /usr/bin/curl "$@"\n' > "$HOME/wrap/curl" && chmod +x "$HOME/wrap/curl"
PATH="$HOME/wrap:$PATH"
NVM_AUTH_HEADER='Bearer ab+c/d=e~f_g.h-i;$(id)`x`' nvm install --no-progress 24.15.0 >/dev/null 2>&1; echo "rc=$?"
sort -u "$HOME/headers"
