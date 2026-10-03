# id: INS-29a
# FreeBSD and OpenBSD have no binaries and always build from source.
# setup: mkdir "$HOME/fake" && printf '#!/bin/sh\nif [ "$1" = -a ]; then echo "$FAKE_OS host 1.0 amd64"; else /usr/bin/uname "$@"; fi\n' > "$HOME/fake/uname" && chmod +x "$HOME/fake/uname"
PATH="$HOME/fake:$PATH"
export NVM_MAKE_JOBS=1
FAKE_OS=FreeBSD nvm install --no-progress 24.15.0; echo "rc=$?"
FAKE_OS=OpenBSD nvm install --no-progress 24.15.0; echo "rc=$?"
