# id: INS-18
# A reported success that fails post-install validation exits 1 without activating.
# setup: printf '#!/bin/sh\nmkdir -p "$5/bin" && printf "#!/bin/sh\\necho $1\\n" > "$5/bin/node" && chmod +x "$5/bin/node" && ln -s missing "$5/bin/npm"\n' > "$HOME/hook" && chmod +x "$HOME/hook"
NVM_INSTALL_THIRD_PARTY_HOOK="$HOME/hook" nvm install 24.15.0; echo "rc=$?"
nvm current
