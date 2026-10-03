# id: DSP-20
# ls takes the first non-empty positional as the pattern and strips one leading v.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
nvm ls "" 20 22
echo "rc=$?"
nvm ls v22
echo "rc=$?"
nvm ls vv22
