# id: HLP-01b
# covers: nvm_echo nvm_find_up
# nvm_find_up prints an empty line when the file is not found.
nvm_find_up no-such-file-here
echo "rc=$?"
