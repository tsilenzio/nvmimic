# id: INS-29b
# Non-WSL Windows cannot build from source, exit 87.
# setup: mkdir "$HOME/fake" && printf '#!/bin/sh\nif [ "$1" = -a ]; then echo "MINGW64_NT-10.0 host 3.4 x86_64 Msys"; else /usr/bin/uname "$@"; fi\n' > "$HOME/fake/uname" && chmod +x "$HOME/fake/uname"
PATH="$HOME/fake:$PATH"
export NVM_MAKE_JOBS=1
nvm install -s 24.15.0; echo "rc=$?"
