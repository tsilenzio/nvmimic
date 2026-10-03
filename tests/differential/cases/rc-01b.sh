# id: RC-01b
# Discovery walks the logical PWD through a symlinked directory.
# setup: nvm install 22.20.0
# setup: nvm install 20.20.2
# setup: mkdir -p real/proj logical && echo 20.20.2 > real/.nvmrc && echo 22.20.0 > logical/.nvmrc
# setup: ln -s ../real/proj logical/link
cd logical/link || exit 99
nvm_find_nvmrc; echo "rc=$?"
nvm which
