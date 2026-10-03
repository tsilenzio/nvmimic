# id: UNI-03
# uninstall refuses the active version with the inferred-from message.
# setup: nvm install 20.20.2
nvm uninstall 20
echo "rc=$?"
nvm uninstall v20.20.2
