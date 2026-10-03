# id: DSP-21
# ls-remote flag handling and first positional pattern.
nvm ls-remote --foo
echo "rc=$?"
nvm ls-remote -- --no-colors 20 18
