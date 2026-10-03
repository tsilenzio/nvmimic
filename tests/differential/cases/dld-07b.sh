# id: DLD-07b
# A missing binary archive with -b prints the abort line and exits 2.
# setup: mkdir "$HOME/fake" && printf '#!/bin/sh\nif [ "$1" = -a ]; then echo "Linux host 6.1 x86_64 GNU/Linux"; elif [ "$1" = -m ]; then echo x86_64; else /usr/bin/uname "$@"; fi\n' > "$HOME/fake/uname" && chmod +x "$HOME/fake/uname"
PATH="$HOME/fake:$PATH"
nvm install -b --no-progress 24.15.0; echo "rc=$?"
