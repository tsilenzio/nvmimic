# id: INS-08
# Without curl or wget install exits 1 unless --offline.
# setup: nvm install 24.15.0
# setup: mkdir "$HOME/tools" && ln -s /usr/bin/* /bin/* "$HOME/tools/" ; rm -f "$HOME/tools/curl" "$HOME/tools/wget"
PATH="${PATH%%:*}:$HOME/tools"
nvm install 20.20.2; echo "rc=$?"
nvm install --offline 24.15.0; echo "rc=$?"
