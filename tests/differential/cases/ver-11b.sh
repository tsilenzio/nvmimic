# id: VER-11b
# Uppercase LTS names fail, with the lowercase error where nvm surfaces it.
# setup: nvm install 22.20.0
nvm version lts/Jod; echo "rc=$?"
nvm which lts/JOD; echo "rc=$?"
nvm alias lts/Jod; echo "rc=$?"
nvm install lts/Jod
