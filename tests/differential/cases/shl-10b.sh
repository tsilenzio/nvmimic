# id: SHL-10b
# use replaces the existing nvm entry in place when it is not behind a system dir.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
mkdir -p "$HOME/tools"
nvm use 24.15.0 >/dev/null
PATH="$HOME/tools:$PATH:/opt/tail"
nvm use 20.20.2 >/dev/null
echo "$PATH"
