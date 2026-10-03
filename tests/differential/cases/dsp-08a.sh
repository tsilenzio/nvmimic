# id: DSP-08a
# A trailing --default is not an install flag and leaves the default alias alone.
# setup: nvm install 22.20.0
nvm install 20.20.2 --default >/dev/null 2>&1
echo "rc=$?"
nvm alias default
