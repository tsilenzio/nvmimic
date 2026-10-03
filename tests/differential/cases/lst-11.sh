# id: LST-11
# ls-remote LTS labels and padding for current, installed and remote lines.
# setup: nvm install 24.21.0
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
nvm ls-remote 24; echo "rc=$?"
nvm ls-remote 20; echo "rc=$?"
nvm ls-remote 26
