# id: SHL-09a
# Under set -e a failing nvm call returns its status and errexit stays on.
# setup: nvm install 24.15.0
set -e
nvm use 24.15.0
nvm ls
nvm current
nvm use 99.0.0 || echo "rc=$?"
case $- in *e*) echo "errexit on" ;; *) echo "errexit off" ;; esac
