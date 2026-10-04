# id: HLP-01a
# covers: nvm_echo nvm_find_up
# nvm_find_up prints the nearest directory containing the file.
mkdir -p a/b/c
touch a/marker
cd a/b/c
nvm_find_up marker
echo "rc=$?"
