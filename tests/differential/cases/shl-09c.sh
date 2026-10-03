# id: SHL-09c
# Under set -a nvm exports nothing beyond its documented variables and allexport stays on.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
env | cut -d= -f1 | sort > "$HOME/env-before"
set -a
nvm use 24.15.0
nvm ls >/dev/null
nvm current
set +a
env | cut -d= -f1 | sort > "$HOME/env-after"
comm -13 "$HOME/env-before" "$HOME/env-after"
set -a
nvm use 20.20.2 >/dev/null
case $- in *a*) echo "allexport on" ;; *) echo "allexport off" ;; esac
