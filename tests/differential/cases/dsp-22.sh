# id: DSP-22
# which scans all arguments and the last version wins.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
nvm which --silent 22.20.0 -- 20.20.2
echo "rc=$?"
nvm which 20.20.2 --foo
