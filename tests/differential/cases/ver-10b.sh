# id: VER-10b
# lts/* is N/A when only an older patch of the LTS line is installed.
# setup: nvm install 24.15.0
nvm version 'lts/*'; echo "rc=$?"
nvm which 'lts/*'
