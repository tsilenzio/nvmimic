# id: SHL-09d
# Under bash set -E nvm works and errtrace stays on.
# shells: bash
# setup: nvm install 24.15.0
set -E
nvm use 24.15.0
nvm current
case $- in *E*) echo "errtrace on" ;; *) echo "errtrace off" ;; esac
