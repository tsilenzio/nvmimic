# id: VER-17a
# Aliases chain through other aliases.
# setup: nvm install 22.20.0
# setup: nvm alias inner 22.20.0
# setup: nvm alias outer inner
nvm version outer; echo "rc=$?"
nvm which outer; echo "rc=$?"
nvm alias outer
