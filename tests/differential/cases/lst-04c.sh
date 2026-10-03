# id: LST-04c
# ls patterns with a comment or newline are parsed as .nvmrc content.
# setup: nvm install 22.20.0
# setup: nvm install 24.21.0
nvm ls '22 # pinned'; echo "rc=$?"
nvm ls "$(printf '# header\n24')"; echo "rc=$?"
nvm ls "$(printf '22\n24')"
