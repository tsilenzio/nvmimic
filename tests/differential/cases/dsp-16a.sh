# id: DSP-16a
# exec rejects an unknown --flag with exit 55.
# setup: nvm install 20.20.2
# covers: nvm nvm:exec nvm_echo nvm_err
nvm exec --foo 20.20.2 node
