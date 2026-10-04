# id: HLP-03b
# README bash cdnvm hook installs an .nvmrc version that is missing.
# shells: bash
# setup: (nvm install 24.15.0)
# covers: nvm nvm:current nvm:deactivate nvm:install nvm:ls nvm:use nvm:version
# covers: nvm_acquire_install_lock nvm_add_iojs_prefix nvm_alias nvm_alias_path nvm_binary_available
# covers: nvm_cache_dir nvm_change_path nvm_compare_checksum nvm_compute_checksum
# covers: nvm_curl_libz_support nvm_curl_use_compression nvm_curl_version nvm_die_on_prefix
# covers: nvm_download nvm_download_artifact nvm_echo nvm_ensure_default_set
# covers: nvm_ensure_version_installed nvm_ensure_version_prefix nvm_err nvm_extract_tarball
# covers: nvm_find_project_dir nvm_find_up nvm_get_arch nvm_get_artifact_compression
# covers: nvm_get_checksum nvm_get_checksum_alg nvm_get_checksum_binary nvm_get_colors
# covers: nvm_get_default_packages nvm_get_download_slug nvm_get_mirror nvm_get_os nvm_grep nvm_has
# covers: nvm_has_colors nvm_has_executable nvm_has_non_aliased nvm_has_system_iojs
# covers: nvm_has_system_node nvm_hash_reset nvm_install_binary nvm_install_binary_extract
# covers: nvm_install_default_packages nvm_install_lock_name nvm_install_npm_if_needed
# covers: nvm_iojs_prefix nvm_is_alias nvm_is_iojs_version nvm_is_merged_node_version
# covers: nvm_is_version_installed nvm_is_zsh nvm_ls nvm_ls_current nvm_ls_remote
# covers: nvm_ls_remote_index_tab nvm_ls_remote_iojs nvm_make_alias nvm_match_version
# covers: nvm_node_prefix nvm_normalize_lts nvm_npmrc_bad_news_bears nvm_num_version_groups
# covers: nvm_print_alias_file nvm_print_color_code nvm_print_npm_version nvm_print_versions
# covers: nvm_release_install_lock nvm_remote_version nvm_remote_versions nvm_resolve_alias
# covers: nvm_resolve_local_alias nvm_strip_iojs_prefix nvm_strip_path nvm_supports_xz
# covers: nvm_tree_contains_path nvm_use_if_needed nvm_validate_implicit_alias nvm_validate_install
# covers: nvm_version nvm_version_dir nvm_version_greater nvm_version_greater_than_or_equal_to
# covers: nvm_version_path
mkdir proj
echo 22.20.0 > proj/.nvmrc
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
cd proj 2>/dev/null
node --version
