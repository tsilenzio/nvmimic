# id: HLP-04a
# README zsh load-nvmrc hook switches on chpwd and reverts to default.
# shells: zsh
# setup: (nvm install 24.15.0)
# setup: (nvm install 20.20.2)
# covers: nvm nvm:use nvm:version nvm_alias nvm_alias_path nvm_change_path nvm_die_on_prefix
# covers: nvm_echo nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_find_nvmrc
# covers: nvm_find_project_dir nvm_find_up nvm_get_os nvm_grep nvm_has nvm_hash_reset
# covers: nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh nvm_ls
# covers: nvm_ls_current nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_alias_file nvm_print_npm_version
# covers: nvm_process_nvmrc nvm_process_nvmrc_content nvm_rc_version nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_tree_contains_path
# covers: nvm_validate_implicit_alias nvm_version nvm_version_dir nvm_version_greater
# covers: nvm_version_path
mkdir proj
echo 20.20.2 > proj/.nvmrc
# place this after nvm initialization!
autoload -U add-zsh-hook

load-nvmrc() {
  local nvmrc_path
  nvmrc_path="$(nvm_find_nvmrc)"

  if [ -n "$nvmrc_path" ]; then
    local nvmrc_node_version
    nvmrc_node_version=$(nvm version "$(cat "${nvmrc_path}")")

    if [ "$nvmrc_node_version" = "N/A" ]; then
      nvm install
    elif [ "$nvmrc_node_version" != "$(nvm version)" ]; then
      nvm use
    fi
  elif [ -n "$(PWD=$OLDPWD nvm_find_nvmrc)" ] && [ "$(nvm version)" != "$(nvm version default)" ]; then
    echo "Reverting to nvm default version"
    nvm use default
  fi
}

add-zsh-hook chpwd load-nvmrc
load-nvmrc
nvm use default >/dev/null
node --version
cd proj
node --version
cd ..
node --version
