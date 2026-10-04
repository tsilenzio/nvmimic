# id: DSP-06c
# exec rejects --ltsfoo as an unsupported option.
# setup: nvm install 20.20.2
# covers: nvm nvm:exec nvm_echo nvm_err
nvm exec --ltsfoo node --version
