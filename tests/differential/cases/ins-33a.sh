# id: INS-33a
# default-packages skips comments and blank lines and installs the rest.
# setup: nvm install 20.20.2
# setup: printf '# tools\n\ntypescript\n  # indented comment\n   \n@scope/pkg@1.2.3\nprettier\n' > "$NVM_DIR/default-packages"
nvm install 24.15.0 2>/dev/null; echo "rc=$?"
