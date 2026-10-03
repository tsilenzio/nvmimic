# id: DLD-05
# Download and checksum stderr lines for a fresh install without progress output.
nvm install --no-progress 24.15.0; echo "rc=$?"
nvm install --no-progress 14.21.3; echo "rc=$?"
