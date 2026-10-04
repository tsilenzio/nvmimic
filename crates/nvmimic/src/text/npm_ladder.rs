//! The lines `nvm install-latest-npm` prints while it works out which npm a
//! Node version can run, copied exactly from nvm.sh.
//
// Text in this file is from nvm (https://github.com/nvm-sh/nvm), MIT licensed:
// Copyright (c) 2010 Tim Caswell, Copyright (c) 2014 Jordan Harband.

// nvm: nvm_install_latest_npm
pub const NODE_0_6: &str = "* `node` v0.6.x can only upgrade to `npm` v1.3.x";
pub const NPM_1: &str =
    "* `npm` v1.x needs to first jump to `npm` v1.4.28 to be able to upgrade further";
pub const NPM_2: &str =
    "* `npm` v2.x needs to first jump to the latest v2 to be able to upgrade further";
pub const STUCK: &str = "* node v0.6 and v0.9 are unable to upgrade further";
pub const BELOW_1_1: &str =
    "* `npm` v4.5.x is the last version that works on `node` versions < v1.1.0";
pub const BELOW_4: &str = "* `npm` v5 and higher do not work on `node` versions below v4.0.0";
pub const NPM_5_3: &str = "* `npm` `v5.3.x` is the last version that works on `node` 4.x versions below v4.4, or 5.x versions below v5.10, due to `Buffer.alloc`";
pub const NPM_5_4_1: &str =
    "* `npm` `v5.4.1` is the last version that works on `node` `v4.5` and `v4.6`";
pub const NPM_5: &str = "* `npm` `v5.x` is the last version that works on `node` below `v6.0.0`";
pub const NPM_6_9: &str = "* `npm` `v6.9` is the last version that works on `node` `v6.0.x`, `v6.1.x`, `v9.0.x`, `v9.1.x`, or `v9.2.x`";
pub const NPM_4_FIRST: &str = "* `npm` `v4.4.4` or later is required to install npm v6.14.18";
pub const NPM_6: &str = "* `npm` `v6.x` is the last version that works on `node` below `v10.0.0`";
pub const NPM_7: &str = "* `npm` `v7.x` is the last version that works on `node` `v13`, `v15`, below `v12.13`, or `v14.0` - `v14.15`";
pub const NPM_8_6: &str = "* `npm` `v8.6` is the last version that works on `node` `v12`, `v14.13` - `v14.16`, or `v16.0` - `v16.12`";
pub const NPM_9: &str = "* `npm` `v9.x` is the last version that works on `node` `< v18.17`, `v19`, or `v20.0` - `v20.4`";
pub const NPM_10: &str = "* `npm` `v10.x` is the last version that works on `node` `< v20.17`, `v21`, or `v22.0` - `v22.8`";
pub const NPM_LATEST: &str =
    "* Installing latest `npm`; if this does not work on your node version, please report a bug!";
