# id: DSP-11a
# use takes the last version argument, honors --silent and ignores unknown --flags.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
nvm use 22.20.0 --foo --silent 20.20.2
echo "rc=$?"
nvm current
