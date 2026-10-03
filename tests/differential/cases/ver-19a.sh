# id: VER-19a
# version-remote resolution for numeric, implicit, LTS and alias specs.
nvm version-remote 24; echo "rc=$?"
nvm version-remote 24.15; echo "rc=$?"
nvm version-remote 2; echo "rc=$?"
nvm version-remote node; echo "rc=$?"
nvm version-remote stable; echo "rc=$?"
nvm version-remote 'lts/*'; echo "rc=$?"
nvm version-remote lts/iron; echo "rc=$?"
nvm version-remote default; echo "rc=$?"
nvm version-remote 99
