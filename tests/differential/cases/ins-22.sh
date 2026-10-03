# id: INS-22
# --no-progress and NVM_NO_PROGRESS=1 hide the progress bar, a set NVM_NO_PROGRESS overrides the flag.
progress() { if grep -q '%$' "$HOME/err"; then echo "progress=yes"; else echo "progress=no"; fi; }
(nvm install 24.15.0 >/dev/null 2>"$HOME/err"; echo "rc=$?"); progress
(nvm install --no-progress 20.20.2 >/dev/null 2>"$HOME/err"; echo "rc=$?"); progress
(export NVM_NO_PROGRESS=1; nvm install 22.20.0 >/dev/null 2>"$HOME/err"; echo "rc=$?"); progress
(export NVM_NO_PROGRESS=0; nvm install --no-progress 18.20.8 >/dev/null 2>"$HOME/err"; echo "rc=$?"); progress
