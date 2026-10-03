# id: DLD-01a
# Invalid NVM_NODEJS_ORG_MIRROR values are rejected with the URL-only message.
for m in 'https://example.com/a b' "https://example.com/'x" 'https://example.com/$(id)' 'ftp://example.com/dist' 'https://example.com/dist?x=1' 'example.com/dist'; do
  NVM_NODEJS_ORG_MIRROR="$m" nvm ls-remote 24.15.0; echo "rc=$?"
done
