# id: DSP-07c
# Recognized leading flags are consumed before the version.
nvm install --no-progress -b 20.20.2 >/dev/null 2>&1
echo "rc=$?"
nvm current
