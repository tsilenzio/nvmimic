# id: SHL-09b
# Under set -e an unguarded failing nvm call aborts the caller with its status.
# setup: nvm install 24.15.0
set -e
nvm ls 99.0.0
echo "not reached"
