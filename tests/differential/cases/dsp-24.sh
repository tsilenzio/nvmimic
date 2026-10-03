# id: DSP-24
# alias flag handling, positionals and the empty-target unalias.
# setup: nvm install 20.20.2
nvm alias --foo
echo "rc=$?"
nvm alias -- --no-colors myalias 20.20.2 ignored
echo "rc=$?"
nvm alias myalias ""
