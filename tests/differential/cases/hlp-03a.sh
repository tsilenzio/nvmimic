# id: HLP-03a
# README bash cdnvm hook switches to .nvmrc on cd and back to default.
# shells: bash
# setup: (nvm install 24.15.0)
# setup: (nvm install 20.20.2)
# covers: nvm nvm:current nvm:deactivate nvm:ls nvm:use nvm:version nvm_alias nvm_alias_path
# covers: nvm_change_path nvm_die_on_prefix nvm_echo nvm_ensure_version_installed
# covers: nvm_ensure_version_prefix nvm_err nvm_find_project_dir nvm_find_up nvm_get_colors
# covers: nvm_get_os nvm_grep nvm_has nvm_has_colors nvm_has_system_iojs nvm_has_system_node
# covers: nvm_hash_reset nvm_iojs_prefix nvm_is_iojs_version nvm_is_version_installed nvm_is_zsh
# covers: nvm_ls nvm_ls_current nvm_match_version nvm_node_prefix nvm_normalize_lts
# covers: nvm_npmrc_bad_news_bears nvm_num_version_groups nvm_print_alias_file nvm_print_color_code
# covers: nvm_print_npm_version nvm_print_versions nvm_resolve_alias nvm_resolve_local_alias
# covers: nvm_strip_iojs_prefix nvm_strip_path nvm_tree_contains_path nvm_validate_implicit_alias
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_path
mkdir proj
echo 20.20.2 > proj/.nvmrc
shopt -s expand_aliases
cdnvm() {
    command cd "$@" || return $?
    nvm_path="$(nvm_find_up .nvmrc | command tr -d '\n')"

    # If there are no .nvmrc file, use the default nvm version
    if [[ ! $nvm_path = *[^[:space:]]* ]]; then

        declare default_version
        default_version="$(nvm version default)"

        # If there is no default version, set it to `node`
        # This will use the latest version on your machine
        if [ $default_version = 'N/A' ]; then
            nvm alias default node
            default_version=$(nvm version default)
        fi

        # If the current version is not the default version, set it to use the default version
        if [ "$(nvm current)" != "${default_version}" ]; then
            nvm use default
        fi
    elif [[ -s "${nvm_path}/.nvmrc" && -r "${nvm_path}/.nvmrc" ]]; then
        declare nvm_version
        nvm_version=$(<"${nvm_path}"/.nvmrc)

        declare locally_resolved_nvm_version
        # `nvm ls` will check all locally-available versions
        # If there are multiple matching versions, take the latest one
        # Remove the `->` and `*` characters and spaces
        # `locally_resolved_nvm_version` will be `N/A` if no local versions are found
        locally_resolved_nvm_version=$(nvm ls --no-colors "${nvm_version}" | command tail -1 | command tr -d '\->*' | command tr -d '[:space:]')

        # If it is not already installed, install it
        # `nvm install` will implicitly use the newly-installed version
        if [ "${locally_resolved_nvm_version}" = 'N/A' ]; then
            nvm install "${nvm_version}";
        elif [ "$(nvm current)" != "${locally_resolved_nvm_version}" ]; then
            nvm use "${nvm_version}";
        fi
    fi
}

alias cd='cdnvm'
cdnvm "$PWD" || exit
node --version
cd proj
node --version
cd ..
node --version
