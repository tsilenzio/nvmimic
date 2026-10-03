# id: SHL-09e
# A non-default IFS does not break nvm and is restored, no helper variables leak.
# setup: nvm install 24.15.0
# setup: nvm install 20.20.2
IFS=,
nvm use 20.20.2
nvm ls
nvm current
[ "$IFS" = , ] && echo "IFS kept"
IFS=' '
for v in DEFAULT_IFS EXIT_CODE VERSION NVM_USE_OUTPUT; do
  eval "[ -n \"\${$v+x}\" ]" && echo "leaked $v"
done
echo done
