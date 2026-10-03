# id: DSP-10
# install with only flags and no .nvmrc proceeds with an empty pattern.
nvm install --save >/dev/null 2>&1
echo "rc=$?"
nvm current
